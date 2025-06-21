use base64::prelude::*;
use sha2::{Digest, Sha256};

pub fn create_hash(parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    let hash = hasher.finalize();
    let truncated = &hash[0..16];
    BASE64_URL_SAFE_NO_PAD.encode(truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hasher() {
        assert_eq!(create_hash(&[]), "47DEQpj8HBSa-_TImW-5JA");
        assert_eq!(create_hash(&[b"abc"]), "ungWv48Bz-pBQUDeXa4iIw");
        assert_eq!(create_hash(&[b"a", b"b"]), "-44g_C5MPySMYMOb1lLzwQ");
    }
}
