
// use std::path::Path;
// use crate::format::{PakError, Result};

// pub struct PakReader {
//     // TODO: Add RawBytesContainer fields
// }

// impl PakReader {
//     pub fn open(_path: impl AsRef<Path>) -> Result<Self> {
//         // TODO: Implement using RawBytesContainer::open_mmap_read
//         todo!("PakReader::open not yet implemented")
//     }
    
//     pub fn get_asset(&self, _name: &str) -> Result<Vec<u8>> {
//         // TODO: Implement asset lookup
//         todo!("PakReader::get_asset not yet implemented")
//     }
    
//     pub fn list_assets(&self) -> Vec<String> {
//         // TODO: Implement asset listing
//         todo!("PakReader::list_assets not yet implemented")
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     #[should_panic(expected = "not yet implemented")]
//     fn test_reader_open() {
//         let _ = PakReader::open("test.pak");
//     }
// }

//! reader.rs - PAK file reader using memory-mapped I/O

use std::path::Path;
use std::collections::HashMap;
use bytemuck_derive::{Pod, Zeroable};

use raw_bytes_container::RawBytesContainer;
use crate::format::{
    PakError, Result,
    PakHeader, TocEntry,
    HEADER_SIZE, TOC_ENTRY_SIZE,
};

/// Reader for PAK files (memory-mapped for zero-copy access)
pub struct PakReader {
    data: RawBytesContainer<u8>,
    header: PakHeader,
    toc: Vec<TocEntry>,
    string_table: Vec<u8>,
    name_map: HashMap<String, usize>, // name -> toc index
}

impl PakReader {
    /// Open a PAK file for reading (memory-mapped)
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        // Memory-map the file
        let data = RawBytesContainer::open_mmap_read(path)
            .map_err(|e| PakError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to mmap PAK file: {}", e)
            )))?;
        
        let slice = data.as_slice();
        
        // Read and validate header
        if slice.len() < HEADER_SIZE {
            return Err(PakError::InvalidToc("File too small".to_string()));
        }
        
        let header = PakHeader::from_bytes(&slice[..HEADER_SIZE])?;
        
        // Read TOC
        let toc_start = header.toc_offset as usize;
        let toc_size = header.entry_count as usize * TOC_ENTRY_SIZE;
        let toc_end = toc_start + toc_size;
        
        if toc_end > slice.len() {
            return Err(PakError::InvalidToc("TOC extends beyond file".to_string()));
        }
        
        let mut toc = Vec::with_capacity(header.entry_count as usize);
        for i in 0..header.entry_count as usize {
            let entry_start = toc_start + i * TOC_ENTRY_SIZE;
            let entry_bytes = &slice[entry_start..entry_start + TOC_ENTRY_SIZE];
            toc.push(TocEntry::from_bytes(entry_bytes)?);
        }
        
        // Read string table
        let string_start = toc_end;
        let string_table = slice[string_start..].to_vec();
        
        // Build name map
        let mut name_map = HashMap::new();
        let mut pos = 0;
        let mut entry_idx = 0;
        
        while pos < string_table.len() && entry_idx < toc.len() {
            if let Some(end) = string_table[pos..].iter().position(|&b| b == 0) {
                if let Ok(name) = std::str::from_utf8(&string_table[pos..pos + end]) {
                    name_map.insert(name.to_string(), entry_idx);
                    entry_idx += 1;
                }
                pos += end + 1;
            } else {
                break;
            }
        }
        
        Ok(Self {
            data,
            header,
            toc,
            string_table,
            name_map,
        })
    }
    
    /// Get an asset by name
    pub fn get_asset(&self, name: &str) -> Result<Vec<u8>> {
        let idx = self.name_map.get(name)
            .ok_or_else(|| PakError::AssetNotFound(name.to_string()))?;
        
        let entry = &self.toc[*idx];
        let slice = self.data.as_slice();
        
        let start = entry.offset as usize;
        let size = if entry.is_compressed() {
            entry.compressed_size as usize
        } else {
            entry.size as usize
        };
        
        let end = start + size;
        if end > slice.len() {
            return Err(PakError::InvalidToc("Asset data extends beyond file".to_string()));
        }
        
        let data = &slice[start..end];
        
        // Decompress if needed
        if entry.is_compressed() {
            #[cfg(feature = "compression")]
            {
                zstd::decode_all(data)
                    .map_err(|e| PakError::DecompressionFailed(e.to_string()))
            }
            #[cfg(not(feature = "compression"))]
            {
                Err(PakError::DecompressionFailed(
                    "Compression support not enabled".to_string()
                ))
            }
        } else {
            Ok(data.to_vec())
        }
    }
    
    /// Get a zero-copy slice to an uncompressed asset
    /// Returns None if asset is compressed
    pub fn get_asset_slice(&self, name: &str) -> Result<Option<&[u8]>> {
        let idx = self.name_map.get(name)
            .ok_or_else(|| PakError::AssetNotFound(name.to_string()))?;
        
        let entry = &self.toc[*idx];
        
        if entry.is_compressed() {
            return Ok(None);
        }
        
        let slice = self.data.as_slice();
        let start = entry.offset as usize;
        let end = start + entry.size as usize;
        
        if end > slice.len() {
            return Err(PakError::InvalidToc("Asset data extends beyond file".to_string()));
        }
        
        Ok(Some(&slice[start..end]))
    }
    
    /// List all asset names
    pub fn list_assets(&self) -> Vec<String> {
        self.name_map.keys().cloned().collect()
    }
    
    /// Get asset metadata
    pub fn get_info(&self, name: &str) -> Option<AssetInfo> {
        let idx = self.name_map.get(name)?;
        let entry = &self.toc[*idx];
        
        Some(AssetInfo {
            name: name.to_string(),
            size: entry.size,
            compressed_size: entry.compressed_size,
            is_compressed: entry.is_compressed(),
            asset_type: crate::format::AssetType::from(entry.type_tag),
        })
    }
    
    /// Get the number of assets in the PAK
    pub fn asset_count(&self) -> usize {
        self.toc.len()
    }
    
    /// Get the PAK header
    pub fn header(&self) -> &PakHeader {
        &self.header
    }
}

/// Asset metadata
#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub name: String,
    pub size: u64,
    pub compressed_size: u64,
    pub is_compressed: bool,
    pub asset_type: crate::format::AssetType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PakBuilder, AssetEntry, AssetType};
    use tempfile::NamedTempFile;

    #[test]
    fn test_reader_open_and_read() -> Result<()> {
        // Create a test PAK file
        let temp = NamedTempFile::new().unwrap();
        let mut builder = PakBuilder::new();
        
        builder.add_asset(AssetEntry::new(
            "test.txt",
            b"Hello, PAK!".to_vec(),
            AssetType::Data
        ));
        
        builder.add_asset(AssetEntry::new(
            "data.bin",
            vec![1, 2, 3, 4, 5],
            AssetType::Data
        ));
        
        builder.build(temp.path())?;
        
        // Read it back
        let reader = PakReader::open(temp.path())?;
        
        assert_eq!(reader.asset_count(), 2);
        
        // Test asset retrieval
        let data = reader.get_asset("test.txt")?;
        assert_eq!(data, b"Hello, PAK!");
        
        let data = reader.get_asset("data.bin")?;
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
        
        Ok(())
    }
    
    #[test]
    fn test_list_assets() -> Result<()> {
        let temp = NamedTempFile::new().unwrap();
        let mut builder = PakBuilder::new();
        
        builder.add_asset(AssetEntry::new("a.txt", vec![1], AssetType::Data));
        builder.add_asset(AssetEntry::new("b.txt", vec![2], AssetType::Data));
        builder.build(temp.path())?;
        
        let reader = PakReader::open(temp.path())?;
        let assets = reader.list_assets();
        
        assert_eq!(assets.len(), 2);
        assert!(assets.contains(&"a.txt".to_string()));
        assert!(assets.contains(&"b.txt".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_asset_not_found() -> Result<()> {
        let temp = NamedTempFile::new().unwrap();
        let builder = PakBuilder::new();
        builder.build(temp.path())?;
        
        let reader = PakReader::open(temp.path())?;
        let result = reader.get_asset("nonexistent.txt");
        
        assert!(matches!(result, Err(PakError::AssetNotFound(_))));
        
        Ok(())
    }
    
    #[test]
    fn test_get_asset_slice() -> Result<()> {
        let temp = NamedTempFile::new().unwrap();
        let mut builder = PakBuilder::new();
        
        builder.add_asset(AssetEntry::new(
            "test.txt",
            b"Zero-copy!".to_vec(),
            AssetType::Data
        ));
        
        builder.build(temp.path())?;
        
        let reader = PakReader::open(temp.path())?;
        
        // Get zero-copy slice
        if let Some(slice) = reader.get_asset_slice("test.txt")? {
            assert_eq!(slice, b"Zero-copy!");
        }
        
        Ok(())
    }
    
    #[test]
    fn test_get_info() -> Result<()> {
        let temp = NamedTempFile::new().unwrap();
        let mut builder = PakBuilder::new();
        
        builder.add_asset(AssetEntry::new(
            "sprite.png",
            vec![0; 1024],
            AssetType::Texture
        ));
        
        builder.build(temp.path())?;
        
        let reader = PakReader::open(temp.path())?;
        let info = reader.get_info("sprite.png").unwrap();
        
        assert_eq!(info.name, "sprite.png");
        assert_eq!(info.asset_type, AssetType::Texture);
        assert_eq!(info.size, 1024);
        
        Ok(())
    }
}