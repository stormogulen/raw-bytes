use bytemuck::{Pod, Zeroable};
use std::mem::size_of;

/// A wrapper around a packed struct of type `T`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Zeroable, Pod)]
pub struct Packed<T: Pod> {
    bytes: [u8; size_of::<T>()],
}

impl<T: Pod> Packed<T> {
    ///  Create  a  new  Packed<T>  from  a  value
    pub fn new(value: T) -> Self {
        Self {
            bytes: bytemuck::bytes_of(&value).try_into().unwrap(),
        }
    }

    ///  Get  a  copy  of  the  inner  value
    pub fn get(&self) -> T {
        bytemuck::from_bytes(&self.bytes).clone()
    }

    ///  Set  the  inner  value
    pub fn set(&mut self, value: T) {
        self.bytes.copy_from_slice(bytemuck::bytes_of(&value));
    }

    ///  Return  bytes  for  raw  access
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    ///  Return  mutable  bytes  for  raw  access
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

///  Slice  helpers
pub fn as_slice<T: Pod>(packed_slice: &[Packed<T>]) -> &[T] {
    bytemuck::cast_slice(packed_slice)
}

pub fn as_mut_slice<T: Pod>(packed_slice: &mut [Packed<T>]) -> &mut [T] {
    bytemuck::cast_slice_mut(packed_slice)
}

#[cfg(test)]
mod tests {

    use super::*;
    #[repr(C, packed)]
    #[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
    struct Packet {
        a: u32,
        b: u16,
        c: u16,
    }

    #[test]
    fn basic_get_set() {
        let p = Packet { a: 1, b: 2, c: 3 };
        let mut packed = Packed::new(p);
        assert_eq!(packed.get(), p);
        let new_val = Packet {
            a: 10,
            b: 20,
            c: 30,
        };
        packed.set(new_val);
        assert_eq!(packed.get(), new_val);
    }

    #[test]
    fn slice_helpers() {
        let packets = [Packet { a: 1, b: 2, c: 3 }, Packet { a: 4, b: 5, c: 6 }];

        let mut packed_arr: Vec<Packed<Packet>> =
            packets.iter().copied().map(Packed::new).collect();
        let slice: &[Packet] = as_slice(&packed_arr);
        assert_eq!(slice[0].a, 1);
        assert_eq!(slice[1].b, 5);
        let slice_mut: &mut [Packet] = as_mut_slice(&mut packed_arr);
        slice_mut[0].a = 42;
        assert_eq!(packed_arr[0].get().a, 42);
    }
}
