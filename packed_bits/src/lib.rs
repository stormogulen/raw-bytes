//! PackedBits: A generic N-bit unsigned integer array packed into bytes.
//! Supports 1 <= N <= 32 and provides safe push/get/set operations.

#[derive(Debug, Clone)]
pub struct PackedBits<const N: usize> {
    data: Vec<u8>,
    len: usize, // number of N-bit elements
}

impl<const N: usize> PackedBits<N> {
    /// Create an empty container
    pub fn new() -> Self {
        assert!(N > 0 && N <= 32, "N must be 1..=32");

        Self {
            data: Vec::new(),
            len: 0,
        }
    }

    /// Push a new value (must fit in N bits)
    pub fn push(&mut self, value: u32) {
        assert!(value < (1 << N), "value must fit in {} bits", N);

        let bit_pos = self.len * N;
        let byte_pos = bit_pos / 8;
        let bit_offset = bit_pos % 8;
        let total_bits = bit_pos + N;
        let required_bytes = (total_bits + 7) / 8;

        if self.data.len() < required_bytes {
            self.data.resize(required_bytes, 0);
        }

        let mut v = value as u64;
        v <<= bit_offset;
        for i in 0..((N + bit_offset + 7) / 8) {
            self.data[byte_pos + i] |= ((v >> (i * 8)) & 0xFF) as u8;
        }

        self.len += 1;
    }

    /// Get the value at index
    pub fn get(&self, index: usize) -> Option<u32> {
        if index >= self.len {
            return None;
        }

        let bit_pos = index * N;
        let byte_pos = bit_pos / 8;
        let bit_offset = bit_pos % 8;
        let mut val: u64 = 0;

        for i in 0..((N + bit_offset + 7) / 8) {
            if byte_pos + i < self.data.len() {
                val |= (self.data[byte_pos + i] as u64) << (i * 8);
            }
        }

        val >>= bit_offset;
        Some((val & ((1 << N) - 1)) as u32)
    }

    /// Set the value at index
    pub fn set(&mut self, index: usize, value: u32) {
        assert!(index < self.len, "index out of bounds");
        assert!(value < (1 << N), "value must fit in {} bits", N);

        let bit_pos = index * N;
        let byte_pos = bit_pos / 8;
        let bit_offset = bit_pos % 8;
        let mut v = value as u64;
        v <<= bit_offset;
        let mask: u64 = ((1u64 << N) - 1) << bit_offset;

        for i in 0..((N + bit_offset + 7) / 8) {
            let byte_mask = ((mask >> (i * 8)) & 0xFF) as u8;
            self.data[byte_pos + i] &= !byte_mask;
            self.data[byte_pos + i] |= ((v >> (i * 8)) & 0xFF) as u8;
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
