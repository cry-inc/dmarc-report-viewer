use serde::Serialize;

#[derive(Serialize)]
pub struct XmlError {
    pub error: String,
    pub xml: String,
}
