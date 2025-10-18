//!  PackedBitsContainer<N>:  A  persistent  bit-packed  array  built  on  RawBytesContainer<u8>.

//!  Supports  in-memory  or  memory-mapped  storage.

use raw_bytes::RawBytesContainer;

///  A  bit-packed  container  using  N  bits  per  element.

pub struct PackedBitsContainer<const N: usize> {
    storage: RawBytesContainer<u8>,
    len: usize, //  number  of  N-bit  elements
}

impl<const N: usize> PackedBitsContainer<N> {
    ///  Create  an  empty  in-memory  container
    pub fn new_in_memory() -> Self {
        assert!(N > 0 && N <= 32, "N  must  be  1..=32");

        Self {
            storage: RawBytesContainer::from_vec(Vec::new()),
            len: 0,
        }
    }

    ///  Create  from  an  existing  RawBytesContainer<u8>
    pub fn from_storage(storage: RawBytesContainer<u8>) -> Self {
        let len_elements = (storage.len() * 8) / N;
        Self {
            storage,
            len: len_elements,
        }
    }

    ///  Access  underlying  storage
    pub fn storage(&self) -> &RawBytesContainer<u8> {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut RawBytesContainer<u8> {
        &mut self.storage
    }

    fn ensure_capacity(&mut self, total_bits: usize) {
        let required_bytes = (total_bits + 7) / 8;

        if let Some(slice) = self.storage.as_slice_mut() {
            if slice.len() < required_bytes {
                self.storage.resize(required_bytes, 0).unwrap();
            }
        }
    }

    ///  Push  a  new  value  (must  fit  in  N  bits)
    pub fn push(&mut self, value: u32) {
        assert!(value < (1 << N), "value  must  fit  in  {}  bits", N);

        let bit_pos = self.len * N;
        self.ensure_capacity(bit_pos + N);
        let byte_pos = bit_pos / 8;
        let bit_offset = bit_pos % 8;
        let mut v = value as u64;
        v <<= bit_offset;

        let slice = self.storage.as_slice_mut().unwrap();
        for i in 0..((N + bit_offset + 7) / 8) {
            slice[byte_pos + i] |= ((v >> (i * 8)) & 0xFF) as u8;
        }

        self.len += 1;
    }

    ///  Get  value  at  index
    pub fn get(&self, index: usize) -> Option<u32> {
        if index >= self.len {
            return None;
        }

        let bit_pos = index * N;
        let byte_pos = bit_pos / 8;
        let bit_offset = bit_pos % 8;
        let mut val: u64 = 0;
        let slice = self.storage.as_slice();

        for i in 0..((N + bit_offset + 7) / 8) {
            if byte_pos + i < slice.len() {
                val |= (slice[byte_pos + i] as u64) << (i * 8);
            }
        }

        val >>= bit_offset;
        Some((val & ((1 << N) - 1)) as u32)
    }

    ///  Set  value  at  index
    pub fn set(&mut self, index: usize, value: u32) {
        assert!(index < self.len, "index  out  of  bounds");
        assert!(value < (1 << N), "value  must  fit  in  {}  bits", N);

        let bit_pos = index * N;
        let byte_pos = bit_pos / 8;
        let bit_offset = bit_pos % 8;

        let mut v = value as u64;
        v <<= bit_offset;

        let mask: u64 = ((1u64 << N) - 1) << bit_offset;
        let slice = self.storage.as_slice_mut().unwrap();
        for i in 0..((N + bit_offset + 7) / 8) {
            let byte_mask = ((mask >> (i * 8)) & 0xFF) as u8;

            slice[byte_pos + i] &= !byte_mask;
            slice[byte_pos + i] |= ((v >> (i * 8)) & 0xFF) as u8;
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

///  Iterator  for  PackedBitsContainer<N>
pub struct PackedBitsContainerIter<'a, const N: usize> {
    container: &'a PackedBitsContainer<N>,
    index: usize,
}

impl<'a, const N: usize> Iterator for PackedBitsContainerIter<'a, N> {
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
}

impl<'a, const N: usize> IntoIterator for &'a PackedBitsContainer<N> {
    type Item = u32;
    type IntoIter = PackedBitsContainerIter<'a, N>;

    fn into_iter(self) -> Self::IntoIter {
        PackedBitsContainerIter {
            container: self,
            index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_bytes::RawBytesContainer;

    #[test]
    fn basic_in_memory() {
        let mut pb = PackedBitsContainer::<12>::new_in_memory();

        pb.push(0xABC);
        pb.push(0x123);
        pb.push(0xFFF);

        assert_eq!(pb.len(), 3);
        assert_eq!(pb.get(0), Some(0xABC));
        assert_eq!(pb.get(1), Some(0x123));
        assert_eq!(pb.get(2), Some(0xFFF));

        pb.set(1, 0x456);
        assert_eq!(pb.get(1), Some(0x456));

        let collected: Vec<_> = (&pb).into_iter().collect();
        assert_eq!(collected, vec![0xABC, 0x456, 0xFFF]);
    }

    #[test]
    fn from_storage() {
        let storage = RawBytesContainer::from_vec(vec![0b1011_1100, 0b0010_0011]);
        let pb = PackedBitsContainer::<4>::from_storage(storage);

        assert_eq!(pb.len(), 4);
        assert_eq!(pb.get(0), Some(0b1100));
    }
}
