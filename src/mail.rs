use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Serialize;

#[derive(Serialize)]
pub struct Mail {
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

    // Set at later stage during parsing
    pub parsing_errors: usize,
}

/// Basic decoder for MIME Encoded Words.
/// Currently only UTF-8 and Base64 are supported.
/// Works only if the whole subject is encoded as a single word.
pub fn decode_subject(value: String) -> String {
    const PREFIX: &str = "=?utf-8?b?";
    const SUFFIX: &str = "?=";
    let lowercase = value.to_lowercase();
    if lowercase.starts_with(PREFIX) && lowercase.ends_with(SUFFIX) {
        let b64 = &value[PREFIX.len()..(value.len() - SUFFIX.len())];
        if let Ok(bytes) = STANDARD.decode(b64) {
            String::from_utf8(bytes).unwrap_or(value)
        } else {
            value
        }
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_subject_test() {
        assert_eq!(decode_subject(String::from("")), "");
        assert_eq!(decode_subject(String::from("basic 123")), "basic 123");
        assert_eq!(decode_subject(String::from("=?utf-8?B??=")), "");
        assert_eq!(decode_subject(String::from("=?utf-8?B?dGV4dA==?=")), "text");
        assert_eq!(decode_subject(String::from("=?utf-8?B?YWJj?=")), "abc");
        assert_eq!(decode_subject(String::from("=?UTF-8?b?YWJj?=")), "abc");
    }
}
