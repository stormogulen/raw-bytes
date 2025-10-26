
pub fn hash_name(name: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in name.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_name() {
        let hash1 = hash_name("test.png");
        let hash2 = hash_name("test.png");
        let hash3 = hash_name("other.png");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}