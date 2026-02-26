use anyhow::{Context, Result, bail};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use regex::Regex;
use serde::Serialize;

#[derive(Serialize)]
pub struct Mail {
    pub id: String,
    pub account: String,
    pub folder: String,
    pub uid: u32,
    pub size: usize,
    pub oversized: bool,
    pub date: i64,
    pub subject: String,
    pub sender: String,
    pub to: String,

    // Body is removed after parsing to save memory
    #[serde(skip)]
    pub body: Option<Vec<u8>>,

    // Set at later stage when extracting the XML files from the body
    pub xml_files: usize,

    // Set at later stage when extracting the JSON files from the body
    pub json_files: usize,

    // Set at later stage during parsing
    pub xml_parsing_errors: usize,

    // Set at later stage during parsing
    pub json_parsing_errors: usize,

    // Hashed IDs of a duplicated DMARC reports found in this mail
    pub dmarc_duplicates: Vec<String>,

    // Hashed IDs of a duplicated SMTP TLS reports found in this mail
    pub tls_duplicates: Vec<String>,
}

/// Decoding of Q-encoded data as described in RFC2047
fn q_decode(mut data: &str) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    while !data.is_empty() {
        if data.starts_with('_') {
            // This is always ASCII space (0x20)
            result.push(0x20);
            data = &data[1..];
        } else if data.starts_with('=') {
            // This is followed by two hex digits encoding a byte
            if data.len() >= 3 {
                let hex = &data[1..3];
                let value = u8::from_str_radix(hex, 16)
                    .context("Expected valid hex string but found something else")?;
                result.push(value);
                data = &data[3..];
            } else {
                bail!("The equal character must be followed by two hex characters");
            }
        } else {
            // Keep everything else as is...
            let byte = &data.as_bytes()[0..1];
            result.extend_from_slice(byte);
            data = &data[1..];
        }
    }
    Ok(result)
}

/// Decoding of MIME encoded words as described in RFC2047
/// This implementation currently only supports UTF-8!
fn decode_word(charset: &str, encoding: &str, data: &str) -> Result<String> {
    let charset = charset.to_lowercase();
    let encoding = encoding.to_lowercase();
    let decoded = if encoding == "b" {
        STANDARD
            .decode(data)
            .context("Failed to decode Base64 data")?
    } else if encoding == "q" {
        q_decode(data).context("Failed to decode Q data")?
    } else {
        bail!("Unsupported encoding: {encoding}")
    };
    if charset == "utf-8" {
        String::from_utf8(decoded).context("Failed to parse UTF-8 string")
    } else {
        // Unsupported charset
        bail!("Unsupported charset: {charset}")
    }
}

/// Basic decoder for subjects containing MIME encoded words.
/// Supported charsets: Only UTF-8
/// Supported encodings: Base64 and Q
pub fn decode_subject(value: &str) -> String {
    let re = Regex::new(r"=\?(.+?)\?(.)\?(.+?)\?=").expect("Failed to parse Regex");
    let mut result = value.to_owned();
    for capture in re.captures_iter(value) {
        let (matched, [charset, encoding, encoded]) = capture.extract();
        let decoded = match decode_word(charset, encoding, encoded) {
            Ok(word) => word,
            Err(_) => continue,
        };
        result = result.replace(matched, &decoded);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q_decode_test() {
        assert_eq!(q_decode("").unwrap(), Vec::<u8>::new());
        assert_eq!(q_decode("abc").unwrap(), vec![b'a', b'b', b'c']);
        assert_eq!(q_decode("_").unwrap(), vec![0x20]);
        assert_eq!(
            q_decode("=00=ff=AA_abc").unwrap(),
            vec![0x00, 0xff, 0xaa, 0x20, b'a', b'b', b'c']
        );
        assert_eq!(
            q_decode("Best=C3=A4tigen").unwrap(),
            vec![66, 101, 115, 116, 195, 164, 116, 105, 103, 101, 110]
        );
    }

    #[test]
    fn decode_word_test() {
        assert_eq!(decode_word("utf-8", "b", "YWJj").unwrap(), "abc");
        assert_eq!(decode_word("UtF-8", "B", "YWJj").unwrap(), "abc");
        assert_eq!(decode_word("utf-8", "q", "=C3=A4").unwrap(), "ä");
        assert_eq!(decode_word("utf-8", "b", "dGV4dA==").unwrap(), "text");

        assert!(decode_word("unknown", "B", "YWJj").is_err());
        assert!(decode_word("utf-8", "unknown", "YWJj").is_err());
        assert!(decode_word("utf-8", "b", "not_valid_b64").is_err());
    }

    #[test]
    fn decode_subject_test() {
        // Can handle empty strings
        assert_eq!(decode_subject(""), "");

        // Can handle strings without encoded words
        assert_eq!(decode_subject("foobar 42"), "foobar 42");

        // Ignores invalid words that cannot be decoded
        assert_eq!(decode_subject("=?foo?z?a?="), "=?foo?z?a?=");

        // Can decode words in the middle
        assert_eq!(decode_subject(" =?UTF-8?b?YWJj?= "), " abc ");

        // Can decode multiple words in one string
        assert_eq!(
            decode_subject(" =?UTF-8?B?YWJj?= =?UTF-8?Q?=C3=A4?= "),
            " abc ä "
        );
    }
}
