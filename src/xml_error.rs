use serde::Serialize;

#[derive(Serialize)]
pub struct XmlError {
    pub mail_uid: u32,
    pub error: String,
    pub xml: String,
}
