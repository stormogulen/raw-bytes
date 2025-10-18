use bytemuck::{Pod, Zeroable};
use packed_structs::Packed;
use raw_bytes::RawBytesContainer;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct PackedStructContainer<T: Pod + Copy> {
    storage: RawBytesContainer<u8>,
    _marker: PhantomData<T>,
}

impl<T: Pod + Copy> PackedStructContainer<T> {
    ///  Create  from  slice  (in-memory)
    pub fn from_slice(data: &[T]) -> Self {
        let bytes = bytemuck::cast_slice(data).to_vec();
        Self {
            storage: RawBytesContainer::from_vec(bytes),
            _marker: PhantomData,
        }
    }

    ///  Open  mmap-backed  file  read-only
    pub fn open_mmap_read<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, raw_bytes::ContainerError> {
        Ok(Self {
            storage: RawBytesContainer::open_mmap_read(path)?,
            _marker: PhantomData,
        })
    }

    ///  Open  mmap-backed  file  read-write
    pub fn open_mmap_rw<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, raw_bytes::ContainerError> {
        Ok(Self {
            storage: RawBytesContainer::open_mmap_rw(path)?,
            _marker: PhantomData,
        })
    }

    ///  Number  of  packed  structs
    pub fn len(&self) -> usize {
        self.storage.as_slice().len() / std::mem::size_of::<T>()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    ///  Access  as  slice  of  T
    pub fn as_slice(&self) -> &[T] {
        bytemuck::cast_slice(self.storage.as_slice())
    }

    ///  Access  as  mutable  slice  if  writable
    pub fn as_slice_mut(&mut self) -> Option<&mut [T]> {
        Some(bytemuck::cast_slice_mut(self.storage.as_slice_mut()?))
    }

    ///  Get  by  index
    pub fn get(&self, index: usize) -> Option<T> {
        self.as_slice().get(index).copied()
    }

    ///  Get  mutable  reference  by  index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.as_slice_mut()?.get_mut(index)
    }

    ///  Append  new  elements  (in-memory  only)
    pub fn append(&mut self, new: &[T]) -> Result<(), raw_bytes::ContainerError> {
        let new_bytes = bytemuck::cast_slice(new);

        self.storage.append(new_bytes)
    }

    ///  Flush  to  disk  (if  mmap  RW)
    pub fn flush(&self) -> Result<(), raw_bytes::ContainerError> {
        self.storage.flush()
    }

    ///  Expose  underlying  storage  (for  advanced  use)
    pub fn storage_mut(&mut self) -> &mut RawBytesContainer<u8> {
        &mut self.storage
    }
}

///  Deref  to  slice  for  ergonomic  access

impl<T: Pod + Copy> Deref for PackedStructContainer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

///  DerefMut  for  mutable  access
impl<T: Pod + Copy> DerefMut for PackedStructContainer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut().expect("Storage  is  not  mutable")
    }
}

///  Iterator  support
impl<'a, T: Pod + Copy> IntoIterator for &'a PackedStructContainer<T> {
    type Item = T;
    type IntoIter = std::iter::Copied<std::slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter().copied()
    }
}

///  Push  multiple  PackedStruct  elements  into  a  PackedStructContainer<T>
#[macro_export]
macro_rules!  push_many  {
                                   ($container:expr,  $($val:expr),*  $(,)?)  =>  {{
                                               $(
                                                           $container.append(&[$val]).unwrap();
                                               )*
                                   }};
                       }

///  Push  a  range  of  PackedStruct  elements  from  any  iterator
#[macro_export]
macro_rules! push_range {
    ($container:expr,  $iter:expr) => {{
        for val in $iter {
            $container.append(&[val]).unwrap();
        }
    }};
}

#[macro_export]
macro_rules!  packed_structs  {
                       ($($val:expr),*  $(,)?)  =>  {{
                                   let  mut  container  =  $crate::PackedStructContainer::<_>::from_slice(&[]);
                                   $(
                                               container.append(&[$val]).unwrap();
                                   )*
                                   container
                       }};
           }
