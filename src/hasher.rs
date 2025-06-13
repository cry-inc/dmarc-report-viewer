use std::hash::{DefaultHasher, Hash, Hasher};

pub fn create_hash(data: &[u8], uid: Option<u32>) -> u32 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    if let Some(uid) = uid {
        uid.hash(&mut hasher);
    }
    hasher.finish() as u32
}
