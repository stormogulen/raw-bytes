//! PackedBytes: A fixed-size byte buffer that can be safely cast to/from Pod types.
//!
//! Useful for serialization, network protocols, or when you need to work with
//! raw bytes but occasionally interpret them as typed data.

use bytemuck::{Pod, Zeroable};
//use packed_struct_types;

/// A fixed-size byte array that can be safely reinterpreted as Pod types.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedBytes<const N: usize> {
    bytes: [u8; N],
}

// Safety: PackedBytes is just a byte array wrapper
unsafe impl<const N: usize> Zeroable for PackedBytes<N> {}
unsafe impl<const N: usize> Pod for PackedBytes<N> {}

impl<const N: usize> Default for PackedBytes<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> PackedBytes<N> {
    /// Create a new PackedBytes filled with zeros.
    pub fn new() -> Self {
        Self { bytes: [0; N] }
    }

    /// Create from a byte array.
    pub fn from_bytes(bytes: [u8; N]) -> Self {
        Self { bytes }
    }

    /// Get a reference to the underlying bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get a mutable reference to the underlying bytes.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    /// Interpret the bytes as a reference to type T.
    ///
    /// # Panics
    /// Panics if T doesn't fit exactly in N bytes.
    pub fn as_pod<T: Pod>(&self) -> &T {
        assert_eq!(
            std::mem::size_of::<T>(),
            N,
            "Type size mismatch: {} bytes required, buffer is {} bytes",
            std::mem::size_of::<T>(),
            N
        );
        bytemuck::from_bytes(&self.bytes)
    }

    /// Interpret the bytes as a mutable reference to type T.
    ///
    /// # Panics
    /// Panics if T doesn't fit exactly in N bytes.
    pub fn as_pod_mut<T: Pod>(&mut self) -> &mut T {
        assert_eq!(
            std::mem::size_of::<T>(),
            N,
            "Type size mismatch: {} bytes required, buffer is {} bytes",
            std::mem::size_of::<T>(),
            N
        );
        bytemuck::from_bytes_mut(&mut self.bytes)
    }

    /// Get a copy of the bytes interpreted as type T.
    ///
    /// # Panics
    /// Panics if T doesn't fit exactly in N bytes.
    pub fn get<T: Pod + Copy>(&self) -> T {
        *self.as_pod::<T>()
    }

    /// Set the bytes from a Pod type.
    ///
    /// # Panics
    /// Panics if T doesn't fit exactly in N bytes.
    pub fn set<T: Pod>(&mut self, value: T) {
        assert_eq!(
            std::mem::size_of::<T>(),
            N,
            "Type size mismatch: {} bytes required, buffer is {} bytes",
            std::mem::size_of::<T>(),
            N
        );
        self.bytes.copy_from_slice(bytemuck::bytes_of(&value));
    }
}

// --- Slice helpers ---

/// Cast a slice of PackedBytes to a slice of Pod types.
///
/// # Panics
/// Panics if T doesn't fit exactly in N bytes.
pub fn cast_slice<T: Pod, const N: usize>(packed_slice: &[PackedBytes<N>]) -> &[T] {
    assert_eq!(
        std::mem::size_of::<T>(),
        N,
        "Type size mismatch: {} bytes required, buffer is {} bytes",
        std::mem::size_of::<T>(),
        N
    );
    bytemuck::cast_slice(packed_slice)
}

/// Cast a mutable slice of PackedBytes to a mutable slice of Pod types.
///
/// # Panics
/// Panics if T doesn't fit exactly in N bytes.
pub fn cast_slice_mut<T: Pod, const N: usize>(packed_slice: &mut [PackedBytes<N>]) -> &mut [T] {
    assert_eq!(
        std::mem::size_of::<T>(),
        N,
        "Type size mismatch: {} bytes required, buffer is {} bytes",
        std::mem::size_of::<T>(),
        N
    );
    bytemuck::cast_slice_mut(packed_slice)
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck_derive::{Pod, Zeroable};

    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
    struct Packet {
        a: u32,
        b: u16,
        c: u16,
    }

    #[test]
    fn basic_get_set() {
        let p = Packet { a: 1, b: 2, c: 3 };
        let mut packed = PackedBytes::<8>::new();
        packed.set(p);
        assert_eq!(packed.get::<Packet>(), p);

        let new_val = Packet {
            a: 10,
            b: 20,
            c: 30,
        };
        packed.set(new_val);
        assert_eq!(packed.get::<Packet>(), new_val);
    }

    #[test]
    fn slice_helpers() {
        let packets = [Packet { a: 1, b: 2, c: 3 }, Packet { a: 4, b: 5, c: 6 }];

        let mut packed_arr: Vec<PackedBytes<8>> = packets
            .iter()
            .map(|&p| {
                let mut buf = PackedBytes::<8>::new();
                buf.set(p);
                buf
            })
            .collect();

        let slice: &[Packet] = cast_slice(&packed_arr);
        assert_eq!(slice[0].a, 1);
        assert_eq!(slice[1].b, 5);

        let slice_mut: &mut [Packet] = cast_slice_mut(&mut packed_arr);
        slice_mut[0].a = 42;
        assert_eq!(packed_arr[0].get::<Packet>().a, 42);
    }

    #[test]
    fn as_bytes() {
        let p = Packet {
            a: 0x12345678,
            b: 0xABCD,
            c: 0xEF01,
        };
        let mut packed = PackedBytes::<8>::new();
        packed.set(p);

        let bytes = packed.as_bytes();
        assert_eq!(bytes.len(), 8);
        assert_eq!(bytes[0], 0x78); // little-endian check
        assert_eq!(bytes[1], 0x56);
    }
}
