
use std::path::Path;
use std::fs;
use crate::format::{AssetType, PakError, Result};

pub struct AssetEntry {
    pub name: String,
    pub data: Vec<u8>,
    pub asset_type: AssetType,
}

impl AssetEntry {
    pub fn new(name: impl Into<String>, data: Vec<u8>, asset_type: AssetType) -> Self {
        Self {
            name: name.into(),
            data,
            asset_type,
        }
    }
    
    pub fn from_file(path: impl AsRef<Path>, asset_type: AssetType) -> Result<Self> {
        let path = path.as_ref();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| PakError::InvalidToc("Invalid filename".to_string()))?
            .to_string();
        
        let data = fs::read(path)?;
        Ok(Self::new(name, data, asset_type))
    }
    
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_entry_new() {
        let entry = AssetEntry::new("test.png", vec![1, 2, 3, 4], AssetType::Texture);
        assert_eq!(entry.name, "test.png");
        assert_eq!(entry.size(), 4);
    }
}
