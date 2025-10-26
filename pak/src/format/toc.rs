
use bytemuck::{Pod, Zeroable};
use bytemuck_derive::{Pod, Zeroable};
use crate::format::constants::{FLAG_COMPRESSED, TOC_ENTRY_SIZE};
use crate::format::error::{PakError, Result};
use crate::format::hash::hash_name;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TocEntry {
    pub name_hash: u64,
    pub offset: u64,
    pub size: u64,
    pub compressed_size: u64,
    pub flags: u32,
    pub type_tag: u32,
}

impl TocEntry {
    pub fn new(name: &str, offset: u64, size: u64, asset_type: AssetType) -> Self {
        Self {
            name_hash: hash_name(name),
            offset,
            size,
            compressed_size: 0,
            flags: 0,
            type_tag: asset_type as u32,
        }
    }
    
    pub fn new_compressed(
        name: &str,
        offset: u64,
        size: u64,
        compressed_size: u64,
        asset_type: AssetType,
    ) -> Self {
        Self {
            name_hash: hash_name(name),
            offset,
            size,
            compressed_size,
            flags: FLAG_COMPRESSED,
            type_tag: asset_type as u32,
        }
    }
    
    pub fn is_compressed(&self) -> bool {
        self.flags & FLAG_COMPRESSED != 0
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < TOC_ENTRY_SIZE {
            return Err(PakError::InvalidToc("TOC entry too small".to_string()));
        }
        Ok(*bytemuck::from_bytes(&bytes[..TOC_ENTRY_SIZE]))
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Unknown = 0,
    Texture = 1,
    Mesh = 2,
    Audio = 3,
    Script = 4,
    Data = 5,
}

impl From<u32> for AssetType {
    fn from(val: u32) -> Self {
        match val {
            1 => AssetType::Texture,
            2 => AssetType::Mesh,
            3 => AssetType::Audio,
            4 => AssetType::Script,
            5 => AssetType::Data,
            _ => AssetType::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toc_entry_size() {
        assert_eq!(std::mem::size_of::<TocEntry>(), TOC_ENTRY_SIZE);
    }
    
    #[test]
    fn test_toc_entry_new() {
        let entry = TocEntry::new("test.png", 1024, 2048, AssetType::Texture);
        
        let offset = entry.offset;
        let size = entry.size;
        
        assert_eq!(offset, 1024);
        assert_eq!(size, 2048);
        assert!(!entry.is_compressed());
    }
}
