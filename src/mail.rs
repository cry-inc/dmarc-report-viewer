use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
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

/// Basic decoder for MIME Encoded Words with UTF8 and Base64
pub fn decode_subject(value: String) -> String {
    const PREFIX: &str = "=?utf-8?B?";
    const SUFFIX: &str = "=?=";
    if value.starts_with(PREFIX) && value.ends_with(SUFFIX) {
        let b64 = value.strip_prefix(PREFIX).expect("Failed to remove prefix");
        let b64 = b64.strip_suffix(SUFFIX).expect("Failed to remove suffix");
        if let Ok(bytes) = STANDARD_NO_PAD.decode(b64) {
            String::from_utf8(bytes).unwrap_or(value)
        } else {
            value
        }
    } else {
        value
    }
}
