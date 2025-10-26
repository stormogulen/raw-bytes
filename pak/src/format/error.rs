
use thiserror::Error;
use std::io;

/// PAK-specific errors
#[derive(Debug, Error)]
pub enum PakError {
    /// Invalid magic bytes in file header
    #[error("Invalid magic bytes (expected 'PAK\\0')")]
    InvalidMagic,
    
    /// Unsupported PAK version
    #[error("Unsupported PAK version: {0}")]
    UnsupportedVersion(u32),
    
    /// Asset not found in archive
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
    
    /// Invalid table of contents
    #[error("Invalid TOC: {0}")]
    InvalidToc(String),
    
    /// Compression error
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    
    /// Decompression error
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
    
    /// IO error wrapper
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Convenience result type
pub type Result<T> = std::result::Result<T, PakError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PakError::InvalidMagic;
        assert_eq!(err.to_string(), "Invalid magic bytes (expected 'PAK\\0')");
        
        let err = PakError::UnsupportedVersion(99);
        assert_eq!(err.to_string(), "Unsupported PAK version: 99");
        
        let err = PakError::AssetNotFound("test.png".to_string());
        assert_eq!(err.to_string(), "Asset not found: test.png");
    }
    
    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let pak_err: PakError = io_err.into();
        
        assert!(matches!(pak_err, PakError::Io(_)));
    }
}
