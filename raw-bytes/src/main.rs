use bytemuck::{Pod, Zeroable}; // traits
use bytemuck_derive::{Pod, Zeroable}; // derive macros

use memmap2::{Mmap, MmapMut};
use std::{
    fs::{File, OpenOptions},
    io,
    ops::{Deref, DerefMut},
    path::Path,
};

/// Storage variants for RawBytesContainer
#[derive(Debug)]
enum Storage<T: Pod> {
    InMemory(Vec<T>),
    MmapRO(Mmap),
    MmapRW(MmapMut),
}

/// Error type for container operations
#[derive(Debug)]
pub enum ContainerError {
    Io(io::Error),
    UnsupportedOperation(&'static str),
}

impl From<io::Error> for ContainerError {
    fn from(err: io::Error) -> Self {
        ContainerError::Io(err)
    }
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerError::Io(err) => write!(f, "IO error: {}", err),
            ContainerError::UnsupportedOperation(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ContainerError {}

/// High-level container for Pod types
#[derive(Debug)]
pub struct RawBytesContainer<T: Pod> {
    storage: Storage<T>,
}

impl<T: Pod> RawBytesContainer<T> {
    /// Create in-memory container from slice
    pub fn from_slice(data: &[T]) -> Self {
        Self {
            storage: Storage::InMemory(data.to_vec()),
        }
    }

    /// Create in-memory container from vec
    pub fn from_vec(data: Vec<T>) -> Self {
        Self {
            storage: Storage::InMemory(data),
        }
    }

    /// Open memory-mapped container (read-only)
    pub fn open_mmap_read<P: AsRef<Path>>(path: P) -> Result<Self, ContainerError> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        if mmap.len() % std::mem::size_of::<T>() != 0 {
            return Err(ContainerError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "File size {} not aligned to type size {}",
                    mmap.len(),
                    std::mem::size_of::<T>()
                ),
            )));
        }

        Ok(Self {
            storage: Storage::MmapRO(mmap),
        })
    }

    /// Open memory-mapped container (read-write)
    pub fn open_mmap_rw<P: AsRef<Path>>(path: P) -> Result<Self, ContainerError> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let mmap = unsafe { MmapMut::map_mut(&file)? };

        if mmap.len() % std::mem::size_of::<T>() != 0 {
            return Err(ContainerError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "File size {} not aligned to type size {}",
                    mmap.len(),
                    std::mem::size_of::<T>()
                ),
            )));
        }

        Ok(Self {
            storage: Storage::MmapRW(mmap),
        })
    }

    /// Immutable view as slice
    pub fn as_slice(&self) -> &[T] {
        match &self.storage {
            Storage::InMemory(vec) => vec,
            Storage::MmapRO(mmap) => bytemuck::cast_slice(mmap),
            Storage::MmapRW(mmap) => bytemuck::cast_slice(&mmap[..]),
        }
    }

    /// Mutable view as slice (returns None for read-only storage)
    pub fn as_slice_mut(&mut self) -> Option<&mut [T]> {
        match &mut self.storage {
            Storage::InMemory(vec) => Some(vec),
            Storage::MmapRW(mmap) => Some(bytemuck::cast_slice_mut(&mut mmap[..])),
            Storage::MmapRO(_) => None,
        }
    }

    /// Append new elements (only works in-memory)
    pub fn append(&mut self, new: &[T]) -> Result<(), ContainerError> {
        match &mut self.storage {
            Storage::InMemory(vec) => {
                vec.extend_from_slice(new);
                Ok(())
            }
            _ => Err(ContainerError::UnsupportedOperation(
                "Append not supported on mmap storage",
            )),
        }
    }

    /// Resize container (only in-memory)
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
                "Resize not supported on mmap storage",
            )),
        }
    }

    /// Write out or flush changes
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
                "Cannot write from read-only mmap",
            )),
        }
    }

    /// Explicit flush for mmap-backed storage
    pub fn flush(&mut self) -> Result<(), ContainerError> {
        match &mut self.storage {
            Storage::MmapRW(mmap) => {
                mmap.flush()?;
                Ok(())
            }
            _ => Err(ContainerError::UnsupportedOperation(
                "Flush only supported on mmap RW",
            )),
        }
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// Check if container is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Allow immutable indexing (`container[0]`)
impl<T: Pod> Deref for RawBytesContainer<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// Allow mutable indexing (`container[0] = ...`) where supported
impl<T: Pod> DerefMut for RawBytesContainer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut().expect("Storage is not mutable")
    }
}

/// Allow passing container into APIs expecting &[T]
impl<T: Pod> AsRef<[T]> for RawBytesContainer<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

/// Allow passing container into APIs expecting &mut [T]
impl<T: Pod> AsMut<[T]> for RawBytesContainer<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_slice_mut().expect("Storage is not mutable")
    }
}

/// Iterator support
impl<'a, T: Pod> IntoIterator for &'a RawBytesContainer<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a, T: Pod> IntoIterator for &'a mut RawBytesContainer<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut()
            .expect("Storage is not mutable")
            .iter_mut()
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::{Pod, Zeroable};

    #[repr(C, packed)]
    #[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
    struct Packet {
        a: u8,
        b: u16,
        c: u32,
    }

    #[test]
    fn test_in_memory_operations() {
        let packets = [Packet { a: 1, b: 2, c: 3 }, Packet { a: 4, b: 5, c: 6 }];

        let mut container = RawBytesContainer::from_slice(&packets);

        assert_eq!(container.len(), 2);
        assert_eq!(container[0].a, 1);

        container.append(&[Packet { a: 7, b: 8, c: 9 }]).unwrap();
        assert_eq!(container.len(), 3);

        container.resize(5, Packet { a: 0, b: 0, c: 0 }).unwrap();
        assert_eq!(container.len(), 5);

        // Iteration
        for p in &container {
            println!("{:?}", p);
        }
    }
}

fn main() -> Result<(), ContainerError> {
    use bytemuck::{Pod, Zeroable};

    #[repr(C, packed)]
    #[derive(Clone, Copy, Debug, Pod, Zeroable)]
    struct Packet {
        a: u8,
        b: u16,
        c: u32,
    }

    // In-memory container
    let packets = [Packet { a: 1, b: 2, c: 3 }, Packet { a: 4, b: 5, c: 6 }];

    let mut container = RawBytesContainer::from_slice(&packets);
    container.append(&[Packet { a: 7, b: 8, c: 9 }])?;
    container.resize(5, Packet { a: 0, b: 0, c: 0 })?;
    container.write_to_file("packets.bin")?;

    println!("Wrote {} packets to file", container.len());

    // Read-only mmap
    let mapped_ro = RawBytesContainer::<Packet>::open_mmap_read("packets.bin")?;
    println!("Read-only: {:?}", mapped_ro.as_slice());

    // Read-write mmap
    let mut mapped_rw = RawBytesContainer::<Packet>::open_mmap_rw("packets.bin")?;
    mapped_rw[0].a = 42; // works with indexing
    mapped_rw.flush()?;
    println!("Read-write: {:?}", mapped_rw.as_slice());

    Ok(())
}
