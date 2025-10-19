//! # PackedBits
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
//! let mut bits = PackedBits::<5>::new();
//! bits.push(31); // max value for 5 bits
//! bits.push(10);
//!
//! assert_eq!(bits.get(0), Some(31));
//! assert_eq!(bits.get(1), Some(10));
//!
//! bits.set(0, 15);
//! assert_eq!(bits.get(0), Some(15));
//!
//! let collected: Vec<u32> = bits.iter().collect();
//! assert_eq!(collected, vec![15, 10]);
//! ```

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedBits<const N: usize> {
    data: Vec<u8>,
    len: usize, // number of N-bit elements
}

impl<const N: usize> PackedBits<N> {
    /// Creates an empty container.
    ///
    /// Panics if `N` is not in the range `1..=32`.
    pub fn new() -> Self {
        assert!(N > 0 && N <= 32, "N must be 1..=32");
        Self {
            data: Vec::new(),
            len: 0,
        }
    }

    /// Creates a container with pre-allocated capacity for `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(N > 0 && N <= 32, "N must be 1..=32");
        let byte_capacity = (capacity * N).div_ceil(8);
        Self {
            data: Vec::with_capacity(byte_capacity),
            len: 0,
        }
    }

    /// Creates a new container from a raw byte buffer and element count.
    ///
    /// Useful for deserialization.
    pub fn from_bytes(data: Vec<u8>, len: usize) -> Self {
        assert!(N > 0 && N <= 32, "N must be 1..=32");
        let min_bytes = (len * N).div_ceil(8);
        assert!(
            data.len() >= min_bytes,
            "insufficient bytes for {} elements",
            len
        );

        Self { data, len }
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
    pub fn push(&mut self, value: u32) {
        let max_val = if N == 32 { u32::MAX } else { (1u32 << N) - 1 };
        assert!(value <= max_val, "value must fit in {} bits", N);

        let bit_pos = self.len * N;
        let byte_pos = bit_pos / 8;
        let bit_offset = bit_pos % 8;
        let total_bits = bit_pos + N;
        let required_bytes = total_bits.div_ceil(8);

        if self.data.len() < required_bytes {
            self.data.resize(required_bytes, 0);
        }

        let mut v = value as u64;
        v <<= bit_offset;

        let num_bytes = (N + bit_offset).div_ceil(8);
        debug_assert!(num_bytes <= 5);

        for i in 0..num_bytes {
            self.data[byte_pos + i] |= ((v >> (i * 8)) & 0xFF) as u8;
        }

        self.len += 1;
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
    pub fn set(&mut self, index: usize, value: u32) {
        assert!(index < self.len, "index out of bounds");

        let max_val = if N == 32 { u32::MAX } else { (1u32 << N) - 1 };
        assert!(value <= max_val, "value must fit in {} bits", N);

        let bit_pos = index * N;
        let byte_pos = bit_pos / 8;
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

        let num_bytes = (N + bit_offset).div_ceil(8);
        debug_assert!(num_bytes <= 5);

        for i in 0..num_bytes {
            if byte_pos + i < self.data.len() {
                let byte_mask = ((mask >> (i * 8)) & 0xFF) as u8;
                self.data[byte_pos + i] &= !byte_mask;
                self.data[byte_pos + i] |= ((v >> (i * 8)) & 0xFF) as u8;
            }
        }
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
        Iter {
            bits: self,
            index: 0,
        }
    }

    /// Extends the container with multiple values.
    pub fn extend_from_slice(&mut self, values: &[u32]) {
        for &v in values {
            self.push(v);
        }
    }
}

impl<const N: usize> Default for PackedBits<N> {
    fn default() -> Self {
        Self::new()
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
        let mut bits = PackedBits::<5>::new();
        bits.push(31);
        bits.push(10);
        bits.push(0);
        assert_eq!(bits.get(0), Some(31));
        assert_eq!(bits.get(1), Some(10));
        assert_eq!(bits.get(2), Some(0));
        assert_eq!(bits.len(), 3);
    }

    #[test]
    fn test_set() {
        let mut bits = PackedBits::<7>::new();
        bits.push(100);
        bits.push(50);
        bits.set(0, 127);
        assert_eq!(bits.get(0), Some(127));
        assert_eq!(bits.get(1), Some(50));
    }

    #[test]
    fn test_n32() {
        let mut bits = PackedBits::<32>::new();
        bits.push(u32::MAX);
        bits.push(12345);
        assert_eq!(bits.get(0), Some(u32::MAX));
        assert_eq!(bits.get(1), Some(12345));
    }

    #[test]
    fn test_iterator() {
        let mut bits = PackedBits::<4>::new();
        bits.push(15);
        bits.push(8);
        bits.push(3);
        let vals: Vec<u32> = bits.iter().collect();
        assert_eq!(vals, vec![15, 8, 3]);
    }

    #[test]
    fn test_n1() {
        let mut bits = PackedBits::<1>::new();
        for i in 0..10 {
            bits.push(i % 2);
        }
        let vals: Vec<u32> = bits.iter().collect();
        assert_eq!(vals, vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1]);
    }

    #[test]
    #[should_panic]
    fn test_overflow() {
        let mut bits = PackedBits::<5>::new();
        bits.push(32); // Should panic
    }

    #[test]
    fn test_extend_and_as_bytes() {
        let mut bits = PackedBits::<3>::new();
        bits.extend_from_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(bits.len(), 5);
        assert!(!bits.as_bytes().is_empty());
    }
}
