use sha2::{Digest, Sha256};

pub fn create_hash(data: &[u8], uid: Option<u32>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    if let Some(uid) = uid {
        hasher.update(uid.to_le_bytes());
    }
    let hash = hasher.finalize();
    format!("{:x}", hash)
}
