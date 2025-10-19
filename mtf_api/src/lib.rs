// mtf_api/src/lib.rs

//! MTF API: High-level interface for Minimal Type Format
//!
//! Provides dynamic reflection and type-safe serialization for MTF-annotated types.

pub use mtf::{MTFError, MTFType, Result};
pub use mtf_derive::MTF;

mod dynamic;
pub use dynamic::{DynamicContainer, FieldHandle};

use std::io::Write;

/// Write a slice of MTF types with embedded metadata.
///
/// Format: [DATA][METADATA_SIZE: u32][METADATA: complete MTF blob]
pub fn write_slice_with_mtf<T: MTFType + bytemuck::Pod>(
    mut out: impl Write,
    slice: &[T],
) -> Result<()> {
    // Write the actual data first
    let raw = bytemuck::cast_slice(slice);
    out.write_all(raw)?;

    // Get the complete MTF blob (includes magic, version, types, strings)
    let blob = T::mtf_type_blob();

    // Write metadata size so readers know where it starts
    let metadata_size = blob.len() as u32;
    out.write_all(&metadata_size.to_le_bytes())?;

    // Write metadata
    out.write_all(blob)?;

    Ok(())
}
