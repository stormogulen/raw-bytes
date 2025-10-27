
// use std::path::Path;
// use crate::asset::AssetEntry;
// use crate::format::{PakError, Result};

// pub struct PakBuilder {
//     assets: Vec<AssetEntry>,
//     compression_level: i32,
//     compress_threshold: usize,
// }

// impl PakBuilder {
//     pub fn new() -> Self {
//         Self {
//             assets: Vec::new(),
//             compression_level: 3,
//             compress_threshold: 512,
//         }
//     }
    
//     pub fn compression_level(&mut self, level: i32) -> &mut Self {
//         self.compression_level = level.clamp(1, 22);
//         self
//     }
    
//     pub fn compress_threshold(&mut self, threshold: usize) -> &mut Self {
//         self.compress_threshold = threshold;
//         self
//     }
    
//     pub fn add_asset(&mut self, asset: AssetEntry) -> &mut Self {
//         self.assets.push(asset);
//         self
//     }
    
//     pub fn build(&self, _output: impl AsRef<Path>) -> Result<()> {
//         // TODO: Implement using RawBytesContainer + PackedStructContainer
//         todo!("PakBuilder::build not yet implemented")
//     }
// }

// impl Default for PakBuilder {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_builder_new() {
//         let builder = PakBuilder::new();
//         assert_eq!(builder.assets.len(), 0);
//     }
// }

//! builder.rs - PAK file builder using raw-bytes containers

use std::path::Path;
use std::collections::HashMap;
use std::fs::File;
//use std::io::Write;
use std::io::{Write, Seek};
use bytemuck_derive::{Pod, Zeroable};

use crate::asset::AssetEntry;
use crate::format::{
    PakError, Result,
    PakHeader, TocEntry, AssetType,
    HEADER_SIZE,
};

/// Builder for creating PAK files
pub struct PakBuilder {
    assets: Vec<AssetEntry>,
    compression_level: i32,
    compress_threshold: usize,
}

impl PakBuilder {
    /// Create a new PAK builder
    pub fn new() -> Self {
        Self {
            assets: Vec::new(),
            compression_level: 3,
            compress_threshold: 512,
        }
    }
    
    /// Set Zstd compression level (1-22, default 3)
    pub fn compression_level(&mut self, level: i32) -> &mut Self {
        self.compression_level = level.clamp(1, 22);
        self
    }
    
    /// Set compression threshold in bytes (default 512)
    /// Assets smaller than this won't be compressed
    pub fn compress_threshold(&mut self, threshold: usize) -> &mut Self {
        self.compress_threshold = threshold;
        self
    }
    
    /// Add an asset to the PAK
    pub fn add_asset(&mut self, asset: AssetEntry) -> &mut Self {
        self.assets.push(asset);
        self
    }
    
    /// Add a directory of assets
    pub fn add_directory(
        &mut self,
        dir: impl AsRef<Path>,
        asset_type: AssetType
    ) -> Result<&mut Self> {
        let dir = dir.as_ref();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let asset = AssetEntry::from_file(entry.path(), asset_type)?;
                self.add_asset(asset);
            }
        }
        Ok(self)
    }

    /// Get the number of assets to be built
    pub fn asset_count(&self) -> usize {
        self.assets.len()
    }
    
    /// Build and write the PAK file
    pub fn build(&self, output: impl AsRef<Path>) -> Result<()> {
        let mut file = File::create(output)?;
        
        // Reserve space for header
        file.write_all(&[0u8; HEADER_SIZE])?;
        
        let data_offset = HEADER_SIZE as u64;
        let mut current_offset = data_offset;
        let mut toc_entries = Vec::new();
        let mut string_table = Vec::new();
        let mut string_offsets = HashMap::new();
        
        // Write asset data and build TOC
        for asset in &self.assets {
            let entry_offset = current_offset;
            let original_size = asset.data.len() as u64;
            
            // Try compression if above threshold
            #[cfg(feature = "compression")]
            let (data_to_write, toc_entry) = if asset.data.len() >= self.compress_threshold {
                match zstd::encode_all(asset.data.as_slice(), self.compression_level) {
                    Ok(compressed) if compressed.len() < asset.data.len() => {
                        // Compression helped
                        let compressed_size = compressed.len() as u64;
                        let entry = TocEntry::new_compressed(
                            &asset.name,
                            entry_offset,
                            original_size,
                            compressed_size,
                            asset.asset_type,
                        );
                        (compressed, entry)
                    }
                    _ => {
                        // Compression didn't help or failed
                        let entry = TocEntry::new(&asset.name, entry_offset, original_size, asset.asset_type);
                        (asset.data.clone(), entry)
                    }
                }
            } else {
                // Too small to compress
                let entry = TocEntry::new(&asset.name, entry_offset, original_size, asset.asset_type);
                (asset.data.clone(), entry)
            };
            
            #[cfg(not(feature = "compression"))]
            let (data_to_write, toc_entry) = {
                let entry = TocEntry::new(&asset.name, entry_offset, original_size, asset.asset_type);
                (asset.data.clone(), entry)
            };
            
            // Write asset data
            file.write_all(&data_to_write)?;
            toc_entries.push(toc_entry);
            
            // Build string table
            if !string_offsets.contains_key(&asset.name) {
                let str_offset = string_table.len();
                string_offsets.insert(asset.name.clone(), str_offset);
                string_table.extend_from_slice(asset.name.as_bytes());
                string_table.push(0); // null terminator
            }
            
            current_offset += data_to_write.len() as u64;
        }
        
        // Write TOC
        let toc_offset = current_offset;
        for entry in &toc_entries {
            file.write_all(entry.as_bytes())?;
        }
        
        // Write string table
        file.write_all(&string_table)?;
        
        // Write header at the beginning
        let header = PakHeader::new(
            toc_entries.len() as u32,
            toc_offset,
            data_offset,
        );
        
        file.seek(std::io::SeekFrom::Start(0))?;
        file.write_all(header.as_bytes())?;
        file.flush()?;
        
        Ok(())
    }
}

impl Default for PakBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = PakBuilder::new();
        assert_eq!(builder.assets.len(), 0);
        assert_eq!(builder.compression_level, 3);
        assert_eq!(builder.compress_threshold, 512);
    }
    
    #[test]
    fn test_builder_settings() {
        let mut builder = PakBuilder::new();
        builder
            .compression_level(10)
            .compress_threshold(1024);
        
        assert_eq!(builder.compression_level, 10);
        assert_eq!(builder.compress_threshold, 1024);
    }
    
    #[test]
    fn test_add_asset() {
        let mut builder = PakBuilder::new();
        builder.add_asset(AssetEntry::new(
            "test.txt",
            b"Hello".to_vec(),
            AssetType::Data
        ));
        
        assert_eq!(builder.assets.len(), 1);
        assert_eq!(builder.assets[0].name, "test.txt");
    }
    
    #[test]
    fn test_build() -> Result<()> {
        use tempfile::NamedTempFile;
        
        let temp = NamedTempFile::new()?;
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
        
        // Verify file was created
        let metadata = std::fs::metadata(temp.path())?;
        assert!(metadata.len() > HEADER_SIZE as u64);
        
        Ok(())
    }
}