use anyhow::{bail, Context, Result};
use dmarc_aggregate_parser::aggregate_report::feedback;
use flate2::read::GzDecoder;
use mailparse::MailHeaderMap;
use std::io::{Cursor, Read};
use tracing::warn;
use zip::ZipArchive;

fn get_xml_from_zip(zip_bytes: &[u8]) -> Result<Vec<u8>> {
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).context("Failed to binary data as ZIP")?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).context("Unable to get file from ZIP")?;
        if !file.name().ends_with(".xml") {
            warn!("{} is not an XML file, skipping...", file.name());
            continue;
        }
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .context("Failed to read XML from ZIP")?;
        return Ok(buffer);
    }
    bail!("Could not find any XML files in ZIP")
}

fn get_xml_from_gz(gz_bytes: &[u8]) -> Result<Vec<u8>> {
    let mut gz = GzDecoder::new(gz_bytes);
    let mut buffer = Vec::new();
    gz.read_to_end(&mut buffer)
        .context("Failed to read file from GZ archive")?;
    Ok(buffer)
}

pub fn extract_reports(mail: &[u8]) -> Result<Vec<feedback>> {
    let parsed = mailparse::parse_mail(mail).context("Failed to parse mail body")?;
    let mut xml_document: Option<Vec<u8>> = None;
    for part in parsed.parts() {
        if let Some(content_type) = part.get_headers().get_first_value("Content-Type") {
            if content_type.contains("application/zip") {
                let body = part
                    .get_body_raw()
                    .context("Failed to get raw body of attachment part")?;
                xml_document = Some(
                    get_xml_from_zip(&body).context("Failed to extract XML from ZIP attachment")?,
                );
            } else if content_type.contains("application/gzip") {
                let body = part
                    .get_body_raw()
                    .context("Failed to get raw body of attachment part")?;
                xml_document = Some(
                    get_xml_from_gz(&body).context("Failed to extract XML from GZ attachment")?,
                );
            }
        }
    }
    let mut reports = Vec::new();
    if let Some(xml) = xml_document {
        let mut cursor = Cursor::new(xml);
        let report = dmarc_aggregate_parser::parse_reader(&mut cursor)
            .context("Failed to parse XML as DMARC report")?;
        reports.push(report);
    }
    Ok(reports)
}
