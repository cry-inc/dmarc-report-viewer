use serde::Serialize;

#[derive(Serialize)]
pub struct Mail {
    pub uid: u32,
    pub size: usize,
    pub date: i64,
    pub subject: String,
    pub sender: String,
    pub to: String,
    #[serde(skip)]
    pub body: Option<Vec<u8>>,
}
