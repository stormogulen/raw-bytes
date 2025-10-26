
use bytemuck::{Pod, Zeroable};
use bytemuck_derive::{Pod, Zeroable};
use crate::format::constants::{PAK_MAGIC, PAK_VERSION, HEADER_SIZE};
use crate::format::error::{PakError, Result};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PakHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub toc_offset: u64,
    pub data_offset: u64,
    pub entry_count: u32,
    pub flags: u32,
}

impl PakHeader {
    pub fn new(entry_count: u32, toc_offset: u64, data_offset: u64) -> Self {
        Self {
            magic: *PAK_MAGIC,
            version: PAK_VERSION,
            toc_offset,
            data_offset,
            entry_count,
            flags: 0,
        }
    }
    
    pub fn validate(&self) -> Result<()> {
        if &self.magic != PAK_MAGIC {
            return Err(PakError::InvalidMagic);
        }
        if self.version != PAK_VERSION {
            return Err(PakError::UnsupportedVersion(self.version));
        }
        Ok(())
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_SIZE {
            return Err(PakError::InvalidToc("Header too small".to_string()));
        }
        let header: PakHeader = *bytemuck::from_bytes(&bytes[..HEADER_SIZE]);
        header.validate()?;
        Ok(header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_size() {
        assert_eq!(std::mem::size_of::<PakHeader>(), HEADER_SIZE);
    }
    
    #[test]
    fn test_header_new() {
        let header = PakHeader::new(10, 1024, 32);
        assert_eq!(&header.magic, PAK_MAGIC);
        
        let version = header.version;
        let entry_count = header.entry_count;
        
        assert_eq!(version, PAK_VERSION);
        assert_eq!(entry_count, 10);
    }
}
