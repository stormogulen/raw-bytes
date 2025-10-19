use crate::{ContainerError, Storage};
use bytemuck::Pod;
use memmap2::{Mmap, MmapMut};
use std::{
    fs::{File, OpenOptions},
    ops::Deref,
    path::Path,
};

///  High-level  container  for  Pod  types
///
///  A  `RawBytesContainer<T>`  can  store  items  in  memory  (`Vec<T>`),
///  or  as  a  memory-mapped  file  (read-only  or  read-write).
#[derive(Debug)]
pub struct RawBytesContainer<T: Pod> {
    storage: Storage<T>,
}

impl<T: Pod> RawBytesContainer<T> {
    ///  Create  a  container  from  a  slice  (clones  data  into  memory).
    pub fn from_slice(data: &[T]) -> Self {
        Self {
            storage: Storage::InMemory(data.to_vec()),
        }
    }

    ///  Create  a  container  from  an  owned  vector.
    pub fn from_vec(data: Vec<T>) -> Self {
        Self {
            storage: Storage::InMemory(data),
        }
    }

    ///  Open  a  read-only  memory-mapped  file.
    pub fn open_mmap_read<P: AsRef<Path>>(path: P) -> Result<Self, ContainerError> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        //  Alignment  check
        if mmap.len() % std::mem::size_of::<T>() != 0 {
            return Err(ContainerError::AlignmentError(format!(
                "File  size  {}  not  aligned  to  type  size  {}",
                mmap.len(),
                std::mem::size_of::<T>()
            )));
        }

        if (mmap.as_ptr() as usize) & (std::mem::align_of::<T>() - 1) != 0 {
            return Err(ContainerError::AlignmentError(format!(
                "Memory  map  address  not  aligned  to  type  alignment  {}",
                std::mem::align_of::<T>()
            )));
        }

        Ok(Self {
            storage: Storage::MmapRO(mmap),
        })
    }

    ///  Open  a  read-write  memory-mapped  file.
    pub fn open_mmap_rw<P: AsRef<Path>>(path: P) -> Result<Self, ContainerError> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let mmap = unsafe { MmapMut::map_mut(&file)? };

        if mmap.len() % std::mem::size_of::<T>() != 0 {
            return Err(ContainerError::AlignmentError(format!(
                "File  size  {}  not  aligned  to  type  size  {}",
                mmap.len(),
                std::mem::size_of::<T>()
            )));
        }

        if (mmap.as_ptr() as usize) % std::mem::align_of::<T>() != 0 {
            return Err(ContainerError::AlignmentError(format!(
                "Memory  map  address  not  aligned  to  type  alignment  {}",
                std::mem::align_of::<T>()
            )));
        }

        Ok(Self {
            storage: Storage::MmapRW(mmap),
        })
    }

    ///  Check  if  this  container  supports  mutation.
    pub fn is_mutable(&self) -> bool {
        matches!(self.storage, Storage::InMemory(_) | Storage::MmapRW(_))
    }

    ///  Get  a  read-only  slice  over  the  data.
    pub fn as_slice(&self) -> &[T] {
        match &self.storage {
            Storage::InMemory(vec) => vec,
            Storage::MmapRO(mmap) => bytemuck::cast_slice(mmap),
            Storage::MmapRW(mmap) => bytemuck::cast_slice(&mmap[..]),
        }
    }

    ///  Get  a  mutable  slice,  if  storage  is  writable.
    pub fn as_slice_mut(&mut self) -> Option<&mut [T]> {
        match &mut self.storage {
            Storage::InMemory(vec) => Some(vec),
            Storage::MmapRW(mmap) => Some(bytemuck::cast_slice_mut(&mut mmap[..])),
            Storage::MmapRO(_) => None,
        }
    }

    ///  Same  as  [`as_slice_mut`],  but  returns  an  error  if  not  mutable.
    pub fn as_slice_mut_checked(&mut self) -> Result<&mut [T], ContainerError> {
        self.as_slice_mut()
            .ok_or(ContainerError::UnsupportedOperation(
                "Cannot  get  mutable  reference  to  read-only  storage",
            ))
    }

    ///  Append  new  items  (only  works  on  in-memory  storage).
    pub fn append(&mut self, new: &[T]) -> Result<(), ContainerError> {
        match &mut self.storage {
            Storage::InMemory(vec) => {
                vec.extend_from_slice(new);
                Ok(())
            }
            _ => Err(ContainerError::UnsupportedOperation(
                "Append  not  supported  on  mmap  storage",
            )),
        }
    }

    ///  Resize  (only  works  on  in-memory  storage).
    pub fn resize(&mut self, new_len: usize, value: T) -> Result<(), ContainerError>
    where
        T: Copy,
    {
        match &mut self.storage {
            Storage::InMemory(vec) => {
                vec.resize(new_len, value);
                Ok(())
            }
            _ => Err(ContainerError::UnsupportedOperation(
                "Resize  not  supported  on  mmap  storage",
            )),
        }
    }

    ///  Write  contents  to  file,  or  flush  mmap  if  writable.
    pub fn write_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ContainerError> {
        match &mut self.storage {
            Storage::InMemory(vec) => {
                std::fs::write(path, bytemuck::cast_slice(vec))?;
                Ok(())
            }
            Storage::MmapRW(mmap) => {
                mmap.flush()?;
                Ok(())
            }
            Storage::MmapRO(_) => Err(ContainerError::UnsupportedOperation(
                "Cannot  write  from  read-only  mmap",
            )),
        }
    }

    ///  Flush  writable  mmap  to  disk.
    pub fn flush(&self) -> Result<(), ContainerError> {
        match &self.storage {
            Storage::MmapRW(mmap) => {
                mmap.flush()?;
                Ok(())
            }
            _ => Err(ContainerError::UnsupportedOperation(
                "Flush  only  supported  on  mmap  RW",
            )),
        }
    }

    ///  Capacity  of  in-memory  storage  (None  for  mmap).
    pub fn capacity(&self) -> Option<usize> {
        match &self.storage {
            Storage::InMemory(vec) => Some(vec.capacity()),
            _ => None,
        }
    }

    ///  Shrink  in-memory  storage  to  fit.
    pub fn shrink_to_fit(&mut self) -> Result<(), ContainerError> {
        match &mut self.storage {
            Storage::InMemory(vec) => {
                vec.shrink_to_fit();
                Ok(())
            }
            _ => Err(ContainerError::UnsupportedOperation(
                "Shrink  only  supported  on  in-memory  storage",
            )),
        }
    }

    ///  Number  of  elements  in  the  container.
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    ///  Returns  true  if  empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    ///  Get  immutable  reference  by  index.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }

    ///  Get  mutable  reference  by  index  (only  if  mutable).
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.as_slice_mut()?.get_mut(index)
    }
}

impl<T: Pod> Deref for RawBytesContainer<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: Pod> AsRef<[T]> for RawBytesContainer<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T: Pod> IntoIterator for &'a RawBytesContainer<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
