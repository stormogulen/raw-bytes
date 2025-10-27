// save/src/save.rs
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use bytemuck::Pod;
use crate::merkle::{MerkleNode, build_merkle_tree};
use packed_struct_container::PackedStructContainer;

/// Save a container with a Merkle root prefix
pub fn save_game<P: AsRef<Path>, T: Pod + Copy>(
    path: P,
    container: &PackedStructContainer<T>,
) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    // Flatten structs into bytes
    let data = bytemuck::cast_slice(container.as_slice());

    // Build Merkle tree for integrity
    let chunks = vec![data.to_vec()]; // Optionally split into smaller blocks
    let root = build_merkle_tree(&chunks);
    let root_hash = root.hash();

    // Write root hash first
    file.write_all(&root_hash)?;

    // Then write the raw struct bytes
    file.write_all(data)?;
    Ok(())
}

/// Load a container and verify the Merkle root
pub fn load_game<P: AsRef<Path>, T: Pod + Copy>(
    path: P,
) -> std::io::Result<PackedStructContainer<T>> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    // Root hash is first 32 bytes
    if bytes.len() < 32 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "corrupt file"));
    }

    let stored_hash = &bytes[..32];
    let data_bytes = &bytes[32..];

    // Compute Merkle root from data
    let chunks = vec![data_bytes.to_vec()];
    let computed_root = build_merkle_tree(&chunks);
    if stored_hash != computed_root.hash().as_slice() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Merkle hash mismatch"));
    }

    // Ensure alignment and convert bytes to T
    if data_bytes.len() % std::mem::size_of::<T>() != 0 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid struct alignment"));
    }

    let structs: &[T] = bytemuck::try_cast_slice(data_bytes)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "cast failed"))?;

    Ok(PackedStructContainer::from_slice(structs))
}
