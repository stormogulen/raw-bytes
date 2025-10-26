
pub const PAK_MAGIC: &[u8; 4] = b"PAK\0";
pub const PAK_VERSION: u32 = 1;
pub const HEADER_SIZE: usize = 32;
pub const TOC_ENTRY_SIZE: usize = 48;
pub const FLAG_COMPRESSED: u32 = 1 << 0;
pub const MAX_NAME_LENGTH: usize = 256;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(PAK_MAGIC, b"PAK\0");
        assert_eq!(PAK_VERSION, 1);
        assert_eq!(HEADER_SIZE, 32);
        assert_eq!(TOC_ENTRY_SIZE, 48);
    }
}