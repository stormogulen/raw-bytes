// //! # PackedBits
//!
//! A generic N-bit unsigned integer array packed tightly into bytes.
//!
//! Efficiently stores integers of any bit width between `1` and `32`, offering
//! safe and compact accessors for `push`, `get`, and `set`.
//!
//! ## Example
//! ```
//! use packed_bits::PackedBits;
//!
//! let mut bits = PackedBits::<5>::new().unwrap();
//! bits.push(31).unwrap(); // max value for 5 bits
//! bits.push(10).unwrap();
//!
//! assert_eq!(bits.get(0), Some(31));
//! assert_eq!(bits.get(1), Some(10));
//!
//! bits.set(0, 15).unwrap();
//! assert_eq!(bits.get(0), Some(15));
//!
//! let collected: Vec<u32> = bits.iter().collect();
//! assert_eq!(collected, vec![15, 10]);
//! ```

//use packed_bits::PackedBitsError;
mod error;
pub use error::PackedBitsError;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedBits<const N: usize> {
    data: Vec<u8>,
    len: usize, // number of N-bit elements
}

impl<const N: usize> PackedBits<N> {
    /// Creates an empty container.
    pub fn new() -> Result<Self, PackedBitsError> {
        if N == 0 || N > 32 {
            return Err(PackedBitsError::InvalidBitWidth(N));
        }
        Ok(Self {
            data: Vec::new(),
            len: 0,
        })
    }

    /// Creates a container with pre-allocated capacity for `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Result<Self, PackedBitsError> {
        if N == 0 || N > 32 {
            return Err(PackedBitsError::InvalidBitWidth(N));
        }
        let byte_capacity = (capacity * N).div_ceil(8);
        Ok(Self {
            data: Vec::with_capacity(byte_capacity),
            len: 0,
        })
    }

    /// Creates a new container from a raw byte buffer and element count.
    ///
    /// Useful for deserialization.
    pub fn from_bytes(data: Vec<u8>, len: usize) -> Result<Self, PackedBitsError> {
        if N == 0 || N > 32 {
            return Err(PackedBitsError::InvalidBitWidth(N));
        }
        let min_bytes = (len * N).div_ceil(8);
        if data.len() < min_bytes {
            return Err(PackedBitsError::InsufficientBytes(len));
        }
        Ok(Self { data, len })
    }

    /// Returns a reference to the underlying byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn into_bytes(self) -> (Vec<u8>, usize) {
        (self.data, self.len)
    }

    /// Reserves capacity for at least `additional` more elements.
    pub fn reserve(&mut self, additional: usize) {
        let total_bits = (self.len + additional) * N;
        let required_bytes = total_bits.div_ceil(8);
        if self.data.capacity() < required_bytes {
            self.data.reserve(required_bytes - self.data.len());
        }
    }

    /// Pushes a new value (must fit in `N` bits).
    pub fn push(&mut self, value: u32) -> Result<(), PackedBitsError> {
        let max_val = if N == 32 { u32::MAX } else { (1u32 << N) - 1 };
        if value > max_val {
            return Err(PackedBitsError::ValueOverflow(value, N));
        }

        let bit_pos = self.len * N;
        let byte_pos = bit_pos / 8;
        let bit_offset = bit_pos % 8;
        let total_bits = bit_pos + N;
        let required_bytes = total_bits.div_ceil(8);

        if self.data.len() < required_bytes {
            self.data.resize(required_bytes, 0);
        }

        let v = (value as u64) << bit_offset;
        let num_bytes = (N + bit_offset).div_ceil(8);
        debug_assert!(num_bytes <= 5);

        for i in 0..num_bytes {
            self.data[byte_pos + i] |= ((v >> (i * 8)) & 0xFF) as u8;
        }

        self.len += 1;
        Ok(())
    }

    /// Gets the value at the specified index.
    pub fn get(&self, index: usize) -> Option<u32> {
        if index >= self.len {
            return None;
        }

        let bit_pos = index * N;
        let byte_pos = bit_pos / 8;
        let bit_offset = bit_pos % 8;

        let mut val: u64 = 0;
        let num_bytes = (N + bit_offset).div_ceil(8);
        debug_assert!(num_bytes <= 5);

        for i in 0..num_bytes {
            if byte_pos + i < self.data.len() {
                val |= (self.data[byte_pos + i] as u64) << (i * 8);
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

    /// Sets the value at the specified index.
    pub fn set(&mut self, index: usize, value: u32) -> Result<(), PackedBitsError> {
        if index >= self.len {
            return Err(PackedBitsError::IndexOutOfBounds(index, self.len));
        }

        let max_val = if N == 32 { u32::MAX } else { (1u32 << N) - 1 };
        if value > max_val {
            return Err(PackedBitsError::ValueOverflow(value, N));
        }

        let bit_pos = index * N;
        let byte_pos = bit_pos / 8;
        let bit_offset = bit_pos % 8;

        let v = (value as u64) << bit_offset;

        let mask: u64 = if N == 32 && bit_offset == 0 {
            u32::MAX as u64
        } else if N + bit_offset >= 64 {
            u64::MAX
        } else {
            ((1u64 << N) - 1) << bit_offset
        };

        let num_bytes = (N + bit_offset).div_ceil(8);
        debug_assert!(num_bytes <= 5);

        for i in 0..num_bytes {
            if byte_pos + i < self.data.len() {
                let byte_mask = ((mask >> (i * 8)) & 0xFF) as u8;
                self.data[byte_pos + i] &= !byte_mask;
                self.data[byte_pos + i] |= ((v >> (i * 8)) & 0xFF) as u8;
            }
        }

        Ok(())
    }

    /// Returns the number of stored elements.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the container has no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clears all elements and resets the buffer.
    pub fn clear(&mut self) {
        self.data.clear();
        self.len = 0;
    }

    /// Returns the capacity in elements before reallocation.
    pub fn capacity(&self) -> usize {
        (self.data.capacity() * 8) / N
    }

    /// Returns an iterator over the packed values.
    pub fn iter(&self) -> Iter<'_, N> {
        Iter { bits: self, index: 0 }
    }

    /// Extends the container with multiple values.
    pub fn extend_from_slice(&mut self, values: &[u32]) -> Result<(), PackedBitsError> {
        for &v in values {
            self.push(v)?;
        }
        Ok(())
    }
}

impl<const N: usize> Default for PackedBits<N> {
    fn default() -> Self {
        Self::new().expect("invalid bit width for PackedBits default")
    }
}

/// Iterator over [`PackedBits`].
pub struct Iter<'a, const N: usize> {
    bits: &'a PackedBits<N>,
    index: usize,
}

impl<'a, const N: usize> Iterator for Iter<'a, N> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.bits.len() {
            let val = self.bits.get(self.index);
            self.index += 1;
            val
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.bits.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, const N: usize> ExactSizeIterator for Iter<'a, N> {}

impl<'a, const N: usize> IntoIterator for &'a PackedBits<N> {
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
    fn test_basic_operations() {
        let mut bits = PackedBits::<5>::new().unwrap();
        bits.push(31).unwrap();
        bits.push(10).unwrap();
        bits.push(0).unwrap();
        assert_eq!(bits.get(0), Some(31));
        assert_eq!(bits.get(1), Some(10));
        assert_eq!(bits.get(2), Some(0));
        assert_eq!(bits.len(), 3);
    }

    #[test]
    fn test_set() {
        let mut bits = PackedBits::<7>::new().unwrap();
        bits.push(100).unwrap();
        bits.push(50).unwrap();
        bits.set(0, 127).unwrap();
        assert_eq!(bits.get(0), Some(127));
        assert_eq!(bits.get(1), Some(50));
    }

    #[test]
    fn test_n32() {
        let mut bits = PackedBits::<32>::new().unwrap();
        bits.push(u32::MAX).unwrap();
        bits.push(12345).unwrap();
        assert_eq!(bits.get(0), Some(u32::MAX));
        assert_eq!(bits.get(1), Some(12345));
    }

    #[test]
    fn test_iterator() {
        let mut bits = PackedBits::<4>::new().unwrap();
        bits.push(15).unwrap();
        bits.push(8).unwrap();
        bits.push(3).unwrap();
        let vals: Vec<u32> = bits.iter().collect();
        assert_eq!(vals, vec![15, 8, 3]);
    }

    #[test]
    fn test_n1() {
        let mut bits = PackedBits::<1>::new().unwrap();
        for i in 0..10 {
            bits.push(i % 2).unwrap();
        }
        let vals: Vec<u32> = bits.iter().collect();
        assert_eq!(vals, vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1]);
    }

    #[test]
    fn test_value_overflow() {
        let mut bits = PackedBits::<5>::new().unwrap();
        let err = bits.push(32).unwrap_err();
        matches!(err, PackedBitsError::ValueOverflow(_, _));
    }

    #[test]
    fn test_extend_and_as_bytes() {
        let mut bits = PackedBits::<3>::new().unwrap();
        bits.extend_from_slice(&[1, 2, 3, 4, 5]).unwrap();
        assert_eq!(bits.len(), 5);
        assert!(!bits.as_bytes().is_empty());
    }
}
