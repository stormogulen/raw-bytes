//! lib.rs - PAK file format library
//! 
//! Minimal archive format for game assets with compression support.

pub mod format;
mod builder;
mod reader;
mod asset;

// Re-export format types
pub use format::{
    error::{PakError, Result},
    constants::*,
    header::PakHeader,
    toc::{TocEntry, AssetType},
    hash::hash_name,
};

// Re-export builders/readers
pub use builder::PakBuilder;
pub use reader::PakReader;
pub use asset::AssetEntry;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imports() {
        // Verify all types are accessible
        let _: PakError;
        let _: Result<()>;
        let _: PakHeader;
        let _: TocEntry;
        let _: AssetType;
        let _: PakBuilder;
        let _: PakReader;
        let _: AssetEntry;
    }
}