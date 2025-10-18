//! PackedBitsContainer<N>: A persistent bit-packed array built on RawBytesContainer<u8>.
//!
//! This container provides bit-packed storage with support for both in-memory and
//! memory-mapped backends via `RawBytesContainer`. It includes metadata persistence
//! for proper serialization/deserialization.
//!
//! # When to use
//!
//! - Use `PackedBits` for simple in-memory bit packing
//! - Use `PackedBitsContainer` when you need:
//!   - Memory-mapped file storage
//!   - Persistent storage with metadata
//!   - Flexible backend (in-memory or mmap)
//!
//! # File format
//!
//! When persisting to storage, the format is:
//! ```text
//! [MAGIC: 4 bytes "PKBT"]
//! [N: u32 (little-endian)]
//! [LEN: u32 (little-endian)]
//! [DATA: variable length bytes]
//! ```

use raw_bytes::RawBytesContainer;

const MAGIC: &[u8; 4] = b"PKBT";
const HEADER_SIZE: usize = 12; // 4 (magic) + 4 (N) + 4 (len)

/// A bit-packed container using N bits per element, backed by RawBytesContainer.
#[derive(Debug)]
pub struct PackedBitsContainer<const N: usize> {
    storage: RawBytesContainer<u8>,
    len: usize, // number of N-bit elements
}

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum PackedBitsError {
    #[error("invalid magic bytes in storage")]
    InvalidMagic,
    
    #[error("N mismatch: expected {expected}, found {found}")]
    InvalidN { expected: usize, found: u32 },
    
    #[error("storage too small for header")]
    StorageTooSmall,
    
    #[error("storage is read-only")]
    StorageReadOnly,
    
    #[error("failed to resize storage")]
    ResizeFailed,
}

type Result<T> = std::result::Result<T, PackedBitsError>;

impl<const N: usize> PackedBitsContainer<N> {
    /// Create an empty in-memory container with metadata header.
    pub fn new_in_memory() -> Self {
        assert!(N > 0 && N <= 32, "N must be 1..=32");

        let mut storage = RawBytesContainer::from_vec(vec![0; HEADER_SIZE]);
        Self::write_header(&mut storage, 0).expect("failed to write header");

        Self { storage, len: 0 }
    }

    /// Create an in-memory container with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(N > 0 && N <= 32, "N must be 1..=32");

        let data_bytes = (capacity * N + 7) / 8;
        let total_bytes = HEADER_SIZE + data_bytes;
        let mut storage = RawBytesContainer::from_vec(vec![0; total_bytes]);
        Self::write_header(&mut storage, 0).expect("failed to write header");

        Self { storage, len: 0 }
    }

    /// Create from an existing RawBytesContainer with header validation.
    pub fn from_storage(storage: RawBytesContainer<u8>) -> Result<Self> {
        assert!(N > 0 && N <= 32, "N must be 1..=32");

        if storage.len() < HEADER_SIZE {
            return Err(PackedBitsError::StorageTooSmall);
        }

        let slice = storage.as_slice();

        // Validate magic
        if &slice[0..4] != MAGIC {
            return Err(PackedBitsError::InvalidMagic);
        }

        // Read N
        let stored_n = u32::from_le_bytes([slice[4], slice[5], slice[6], slice[7]]);
        if stored_n as usize != N {
            return Err(PackedBitsError::InvalidN {
                expected: N,
                found: stored_n,
            });
        }

        // Read len
        let len = u32::from_le_bytes([slice[8], slice[9], slice[10], slice[11]]) as usize;

        Ok(Self { storage, len })
    }

    /// Create from raw storage without header (legacy compatibility).
    pub fn from_storage_raw(storage: RawBytesContainer<u8>) -> Self {
        let len_elements = (storage.len() * 8) / N;
        Self { storage, len: len_elements }
    }

    /// Write header to storage.
    fn write_header(storage: &mut RawBytesContainer<u8>, len: usize) -> Result<()> {
        let slice = storage
            .as_slice_mut()
            .ok_or(PackedBitsError::StorageReadOnly)?;

        if slice.len() < HEADER_SIZE {
            return Err(PackedBitsError::StorageTooSmall);
        }

        slice[0..4].copy_from_slice(MAGIC);
        slice[4..8].copy_from_slice(&(N as u32).to_le_bytes());
        slice[8..12].copy_from_slice(&(len as u32).to_le_bytes());

        Ok(())
    }

    /// Update len in header.
    fn update_len_in_header(&mut self) -> Result<()> {
        let slice = self
            .storage
            .as_slice_mut()
            .ok_or(PackedBitsError::StorageReadOnly)?;

        slice[8..12].copy_from_slice(&(self.len as u32).to_le_bytes());
        Ok(())
    }

    /// Access underlying storage.
    pub fn storage(&self) -> &RawBytesContainer<u8> {
        &self.storage
    }

    /// Mutable access to underlying storage.
    pub fn storage_mut(&mut self) -> &mut RawBytesContainer<u8> {
        &mut self.storage
    }

    /// Ensure storage has enough capacity for the given number of bits.
    fn ensure_capacity(&mut self, total_bits: usize) -> Result<()> {
        let required_bytes = HEADER_SIZE + (total_bits + 7) / 8;

        if self.storage.as_slice().len() < required_bytes {
            self.storage
                .resize(required_bytes, 0)
                .map_err(|_| PackedBitsError::ResizeFailed)?;
        }

        Ok(())
    }

    /// Push a new value (must fit in N bits).
    pub fn push(&mut self, value: u32) -> Result<()> {
        let max_val = if N == 32 {
            u32::MAX
        } else {
            (1u32 << N) - 1
        };
        assert!(value <= max_val, "value must fit in {} bits", N);

        let bit_pos = self.len * N;
        self.ensure_capacity(bit_pos + N)?;

        let byte_pos = HEADER_SIZE + bit_pos / 8;
        let bit_offset = bit_pos % 8;
        let mut v = value as u64;
        v <<= bit_offset;

        let slice = self
            .storage
            .as_slice_mut()
            .ok_or(PackedBitsError::StorageReadOnly)?;

        let num_bytes = (N + bit_offset + 7) / 8;
        debug_assert!(num_bytes <= 5);

        for i in 0..num_bytes {
            slice[byte_pos + i] |= ((v >> (i * 8)) & 0xFF) as u8;
        }

        self.len += 1;
        self.update_len_in_header()?;

        Ok(())
    }

    /// Get value at index.
    pub fn get(&self, index: usize) -> Option<u32> {
        if index >= self.len {
            return None;
        }

        let bit_pos = index * N;
        let byte_pos = HEADER_SIZE + bit_pos / 8;
        let bit_offset = bit_pos % 8;
        let mut val: u64 = 0;
        let slice = self.storage.as_slice();

        let num_bytes = (N + bit_offset + 7) / 8;
        debug_assert!(num_bytes <= 5);

        for i in 0..num_bytes {
            if byte_pos + i < slice.len() {
                val |= (slice[byte_pos + i] as u64) << (i * 8);
            }
        }

        val >>= bit_offset;

        let mask = if N == 32 {
            u32::MAX as u64
        } else {
            (1u64 << N) - 1
        };

        Some((val & mask) as u32)
    }

    /// Set value at index.
    pub fn set(&mut self, index: usize, value: u32) -> Result<()> {
        assert!(index < self.len, "index out of bounds");

        let max_val = if N == 32 {
            u32::MAX
        } else {
            (1u32 << N) - 1
        };
        assert!(value <= max_val, "value must fit in {} bits", N);

        let bit_pos = index * N;
        let byte_pos = HEADER_SIZE + bit_pos / 8;
        let bit_offset = bit_pos % 8;

        let mut v = value as u64;
        v <<= bit_offset;

        let mask: u64 = if N == 32 && bit_offset == 0 {
            u32::MAX as u64
        } else if N + bit_offset >= 64 {
            u64::MAX
        } else {
            ((1u64 << N) - 1) << bit_offset
        };

        let slice = self
            .storage
            .as_slice_mut()
            .ok_or(PackedBitsError::StorageReadOnly)?;

        let num_bytes = (N + bit_offset + 7) / 8;
        debug_assert!(num_bytes <= 5);

        for i in 0..num_bytes {
            if byte_pos + i < slice.len() {
                let byte_mask = ((mask >> (i * 8)) & 0xFF) as u8;
                slice[byte_pos + i] &= !byte_mask;
                slice[byte_pos + i] |= ((v >> (i * 8)) & 0xFF) as u8;
            }
        }

        Ok(())
    }

    /// Returns the number of stored elements.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the container has no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clear all elements (keeps header).
    pub fn clear(&mut self) -> Result<()> {
        self.len = 0;
        self.storage
            .resize(HEADER_SIZE, 0)
            .map_err(|_| PackedBitsError::ResizeFailed)?;
        self.update_len_in_header()?;
        Ok(())
    }

    /// Returns the capacity in elements before reallocation.
    pub fn capacity(&self) -> usize {
        let data_bytes = self.storage.as_slice().len().saturating_sub(HEADER_SIZE);
        (data_bytes * 8) / N
    }

    /// Returns an iterator over the packed values.
    pub fn iter(&self) -> Iter<'_, N> {
        Iter {
            container: self,
            index: 0,
        }
    }
}

/// Iterator for PackedBitsContainer<N>
pub struct Iter<'a, const N: usize> {
    container: &'a PackedBitsContainer<N>,
    index: usize,
}

impl<'a, const N: usize> Iterator for Iter<'a, N> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.container.len() {
            None
        } else {
            let val = self.container.get(self.index);
            self.index += 1;
            val
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.container.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, const N: usize> ExactSizeIterator for Iter<'a, N> {}

impl<'a, const N: usize> IntoIterator for &'a PackedBitsContainer<N> {
    type Item = u32;
    type IntoIter = Iter<'a, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_in_memory() {
        let mut pb = PackedBitsContainer::<12>::new_in_memory();

        pb.push(0xABC).unwrap();
        pb.push(0x123).unwrap();
        pb.push(0xFFF).unwrap();

        assert_eq!(pb.len(), 3);
        assert_eq!(pb.get(0), Some(0xABC));
        assert_eq!(pb.get(1), Some(0x123));
        assert_eq!(pb.get(2), Some(0xFFF));

        pb.set(1, 0x456).unwrap();
        assert_eq!(pb.get(1), Some(0x456));

        let collected: Vec<_> = pb.iter().collect();
        assert_eq!(collected, vec![0xABC, 0x456, 0xFFF]);
    }

    #[test]
    fn test_header_persistence() {
        let mut pb = PackedBitsContainer::<7>::new_in_memory();
        pb.push(100).unwrap();
        pb.push(50).unwrap();

        // Get the raw bytes
        let bytes = pb.storage().as_slice().to_vec();

        // Recreate from new storage with same bytes
        let storage = RawBytesContainer::from_vec(bytes);
        let pb2 = PackedBitsContainer::<7>::from_storage(storage).unwrap();
        assert_eq!(pb2.len(), 2);
        assert_eq!(pb2.get(0), Some(100));
        assert_eq!(pb2.get(1), Some(50));
    }

    #[test]
    fn test_n32() {
        let mut pb = PackedBitsContainer::<32>::new_in_memory();
        pb.push(u32::MAX).unwrap();
        pb.push(12345).unwrap();

        assert_eq!(pb.get(0), Some(u32::MAX));
        assert_eq!(pb.get(1), Some(12345));
    }

    #[test]
    fn test_clear() {
        let mut pb = PackedBitsContainer::<5>::new_in_memory();
        pb.push(10).unwrap();
        pb.push(20).unwrap();
        assert_eq!(pb.len(), 2);

        pb.clear().unwrap();
        assert_eq!(pb.len(), 0);
        assert!(pb.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let mut pb = PackedBitsContainer::<8>::with_capacity(100);
        assert!(pb.capacity() >= 100);
        assert_eq!(pb.len(), 0);

        for i in 0..50 {
            pb.push(i as u32).unwrap();
        }
        assert_eq!(pb.len(), 50);
    }

    #[test]
    fn test_wrong_n() {
        let mut pb = PackedBitsContainer::<7>::new_in_memory();
        pb.push(100).unwrap();

        // Get the raw bytes
        let bytes = pb.storage().as_slice().to_vec();

        // Try to load with wrong N
        let storage = RawBytesContainer::from_vec(bytes);
        let result = PackedBitsContainer::<12>::from_storage(storage);

        assert!(matches!(
            result,
            Err(PackedBitsError::InvalidN { expected: 12, found: 7 })
        ));
    }
}