use crate::hasher::create_hash;
use crate::mail::Mail;
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use mailparse::{MailHeaderMap, ParsedMail};
use std::io::{Cursor, Read};
use tracing::{trace, warn};
use zip::ZipArchive;

/// Get zero or more XML files from a ZIP archive
fn get_xml_from_zip(zip_bytes: &[u8]) -> Result<Vec<Vec<u8>>> {
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).context("Failed to binary data as ZIP")?;

    let file_count = archive.len();
    if file_count == 0 {
        warn!("ZIP file is empty");
    }

    let mut xml_files = Vec::new();
    for i in 0..file_count {
        let mut file = archive.by_index(i).context("Unable to get file from ZIP")?;
        let file_name = file.name();

        if !file_name.ends_with(".xml") {
            warn!("File {file_name} in ZIP is not an XML file, skipping...",);
            continue;
        }

        let mut xml_file = Vec::new();
        file.read_to_end(&mut xml_file)
            .context("Failed to read XML from ZIP")?;
        xml_files.push(xml_file);
    }

    Ok(xml_files)
}

/// Merge name value of content type header in case its split like described
/// here: https://datatracker.ietf.org/doc/html/rfc2231#section-3
/// Please note that `mailparse` already removes the line breaks before this function is used!
fn merge_name_parts(value: &str) -> String {
    let mut out_buffer = String::new();
    let mut name_buffer = String::new();
    let mut next = 0;

    for segment in value.trim().split("; ") {
        let next_prefix = format!("name*{next}=");
        if let Some(mut candidate) = segment.trim().strip_prefix(&next_prefix).map(String::from) {
            next += 1;
            if candidate.ends_with(';') {
                candidate.pop();
            }
            if candidate.len() > 2 && candidate.starts_with('"') && candidate.ends_with('"') {
                candidate.remove(0);
                candidate.pop();
            }
            name_buffer += &candidate;
        } else if out_buffer.is_empty() {
            out_buffer += segment;
        } else {
            out_buffer += &format!("; {segment}");
        }
    }

    if !name_buffer.is_empty() {
        out_buffer += &format!("; name=\"{name_buffer}\"");
    }

    out_buffer
}

/// Get a single XML file from a GZ archive
fn get_xml_from_gz(gz_bytes: &[u8]) -> Result<Vec<u8>> {
    let mut gz = GzDecoder::new(gz_bytes);
    let mut xml_file = Vec::new();
    gz.read_to_end(&mut xml_file)
        .context("Failed to read file from GZ archive")?;
    Ok(xml_file)
}

pub fn extract_xml_files(mail: &mut Mail) -> Result<Vec<XmlFile>> {
    // Consume mail body to avoid keeping the longer needed data in memory
    let body = mail.body.take().context("Missing mail body")?;

    let mut xml_files = Vec::new();
    let parsed = mailparse::parse_mail(&body).context("Failed to parse mail body")?;
    let parts: Vec<&ParsedMail> = parsed.parts().collect();
    let uid = mail.uid;
    trace!("Parsed mail with UID {uid} and found {} parts", parts.len());
    for (index, part) in parts.iter().enumerate() {
        let Some(content_type) = part.get_headers().get_first_value("Content-Type") else {
            trace!("Skipping part {index} of mail with UID {uid} because of missing content type",);
            continue;
        };
        trace!("Part {index} of mail with UID {uid} has content type '{content_type}'");

        // Some long names in content type headers names are split into multiple lines and parts:
        // application/octet-stream;
        //   name*0=amazonses.com!xxxxxxxxxxxxxxxxxxxxxx!1745884800!1745971200.xm;
        //   name*1=l.gz
        let content_type = merge_name_parts(&content_type);

        // Detect compression based on content type header.
        // In most cases is directly a ZIP or GZIP type, but in some cases its generic
        // and we need to check for a file name ending with a certain extension.
        // For example AWS uses such values:
        // Content-Type: application/octet-stream;name=amazonses.com!example.com!1722384000!1722470400.xml.gz
        if content_type.contains("application/zip")
            || content_type.contains("application/octet-stream") && content_type.contains(".zip")
            || content_type.contains("application/x-zip-compressed")
                && content_type.contains(".zip")
        {
            trace!("Detected ZIP attachment for mail with UID {uid} in part {index}");
            let body = part
                .get_body_raw()
                .context("Failed to get raw body of attachment part")?;
            let xml_files_zip =
                get_xml_from_zip(&body).context("Failed to extract XML from ZIP attachment")?;
            trace!(
                "Extracted {} XML files from ZIP in part {index} of mail with UID {uid}",
                xml_files_zip.len()
            );
            for xml in xml_files_zip {
                let hash = create_hash(&xml, Some(mail.uid));
                xml_files.push(XmlFile {
                    data: xml,
                    mail_uid: mail.uid,
                    hash,
                });
            }
        } else if content_type.contains("application/gzip")
            || content_type.contains("application/octet-stream") && content_type.contains(".xml.gz")
        {
            trace!("Detected GZ attachment for mail with UID {uid} in part {index}");
            let body = part
                .get_body_raw()
                .context("Failed to get raw body of attachment part")?;
            let xml = get_xml_from_gz(&body).context("Failed to extract XML from GZ attachment")?;
            let hash = create_hash(&xml, Some(mail.uid));
            xml_files.push(XmlFile {
                data: xml,
                mail_uid: mail.uid,
                hash,
            });
        } else if content_type.contains("text/xml")
            || content_type.contains("application/octet-stream") && content_type.contains(".xml")
        {
            trace!("Detected uncompressed XML attachment for mail with UID {uid} in part {index}");
            let xml = part
                .get_body_raw()
                .context("Failed to get raw body of attachment part")?;
            let hash = create_hash(&xml, Some(mail.uid));
            xml_files.push(XmlFile {
                data: xml,
                mail_uid: mail.uid,
                hash,
            });
        }
    }

    Ok(xml_files)
}

/// In-memory representation of an unparsed XML file with mail UID and hash
pub struct XmlFile {
    /// UID of the mail that contained this XML file
    pub mail_uid: u32,
    /// Binary data of the XML file
    pub data: Vec<u8>,
    /// Hash of the XML data AND mail UID.
    /// UID needs to be included to avoid the same XML file from multiple mails being treated as the same file!
    pub hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_name_parts() {
        let input = "application/octet-stream;  name*0=amazonses.com!xxxxxxxxxxxxxxxxxxxxxx!1745884800!1745971200.xm;  name*1=l.gz";
        let output = merge_name_parts(input);
        assert!(output.contains(
            "name=\"amazonses.com!xxxxxxxxxxxxxxxxxxxxxx!1745884800!1745971200.xml.gz\""
        ));

        let input = "application/octet-stream;  name*0=foo;  name*1=bar;  name*2=.jpeg";
        let output = merge_name_parts(input);
        assert!(output.contains("name=\"foobar.jpeg\""));

        let input = "application/octet-stream; name*0=\"foo\"; name*1=\"bar\"; name*2=\".jpeg\"";
        let output = merge_name_parts(input);
        assert!(output.contains("name=\"foobar.jpeg\""));
    }
}
