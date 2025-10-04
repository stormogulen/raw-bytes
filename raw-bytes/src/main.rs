use bytemuck::Pod;
use bytemuck_derive::{Pod, Zeroable}; // derive macros
use memmap2::{Mmap, MmapMut};
use std::{
    fs::{File, OpenOptions},
    io,
    ops::Deref,
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
    AlignmentError(String),
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
            ContainerError::AlignmentError(msg) => write!(f, "Alignment error: {}", msg),
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

        Self::validate_alignment(&mmap)?;

        Ok(Self {
            storage: Storage::MmapRO(mmap),
        })
    }

    /// Open memory-mapped container (read-write)
    pub fn open_mmap_rw<P: AsRef<Path>>(path: P) -> Result<Self, ContainerError> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Self::validate_alignment(&mmap)?;

        Ok(Self {
            storage: Storage::MmapRW(mmap),
        })
    }

    /// Check container mutability
    pub fn is_mutable(&self) -> bool {
        matches!(self.storage, Storage::InMemory(_) | Storage::MmapRW(_))
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

    /// Mutable view as slice (returns error for read-only storage)
    pub fn as_slice_mut_checked(&mut self) -> Result<&mut [T], ContainerError> {
        self.as_slice_mut()
            .ok_or(ContainerError::UnsupportedOperation(
                "Cannot get mutable reference to read-only storage",
            ))
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
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ContainerError> {
        match &self.storage {
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

    /// Number of elements
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get element at index (checked)
    pub fn get(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }

    /// Get mutable element at index (checked)
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.as_slice_mut()?.get_mut(index)
    }

    /// Get mutable element at index (checked with Result)
    pub fn get_mut_checked(&mut self, index: usize) -> Result<&mut T, ContainerError> {
        self.get_mut(index)
            .ok_or(ContainerError::UnsupportedOperation(
                "Index out of bounds or read-only",
            ))
    }

    /// Safe mutable iterator
    pub fn iter_mut_checked(
        &mut self,
    ) -> Result<impl Iterator<Item = &mut T> + '_, ContainerError> {
        Ok(self.as_slice_mut_checked()?.iter_mut())
    }

    /// Alignment validation helper
    fn validate_alignment(slice: &[u8]) -> Result<(), ContainerError> {
        if slice.len() % std::mem::size_of::<T>() != 0 {
            return Err(ContainerError::AlignmentError(format!(
                "File size {} not aligned to type size {}",
                slice.len(),
                std::mem::size_of::<T>()
            )));
        }
        if (slice.as_ptr() as usize) & (std::mem::align_of::<T>() - 1) != 0 {
            return Err(ContainerError::AlignmentError(format!(
                "Memory map address not aligned to type alignment {}",
                std::mem::align_of::<T>()
            )));
        }
        Ok(())
    }
}

/// Immutable indexing
impl<T: Pod> Deref for RawBytesContainer<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// AsRef support for APIs expecting &[T]
impl<T: Pod> AsRef<[T]> for RawBytesContainer<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

/// Iterator support (immutable)
impl<'a, T: Pod> IntoIterator for &'a RawBytesContainer<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

/// From conversions for ergonomics
impl<T: Pod> From<Vec<T>> for RawBytesContainer<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}

impl<T: Pod> From<&[T]> for RawBytesContainer<T> {
    fn from(slice: &[T]) -> Self {
        Self::from_slice(slice)
    }
}

/// Default empty container
impl<T: Pod> Default for RawBytesContainer<T> {
    fn default() -> Self {
        Self {
            storage: Storage::InMemory(Vec::new()),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::{Pod, Zeroable};
    use tempfile::NamedTempFile;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
    struct Packet {
        a: u32,
        b: u16,
        c: u16,
    }

    #[test]
    fn test_in_memory_operations() {
        let packets = [Packet { a: 1, b: 2, c: 0 }, Packet { a: 4, b: 5, c: 0 }];
        let mut container = RawBytesContainer::from_slice(&packets);

        assert_eq!(container.len(), 2);
        assert_eq!(container[0].a, 1);

        container.get_mut_checked(0).unwrap().a = 42;
        assert_eq!(container[0].a, 42);

        container.append(&[Packet { a: 7, b: 8, c: 0 }]).unwrap();
        assert_eq!(container.len(), 3);

        container.resize(5, Packet { a: 0, b: 0, c: 0 }).unwrap();
        assert_eq!(container.len(), 5);

        for p in &container {
            println!("{:?}", p);
        }

        for p in container.iter_mut_checked().unwrap() {
            p.a += 1;
        }
    }

    #[test]
    fn test_read_only_protection() {
        let packets = [Packet { a: 1, b: 2, c: 0 }];

        // Create temporary file
        let mut tmpfile = NamedTempFile::new().unwrap();
        let tmp_path = tmpfile.path();

        // Write initial in-memory container to temp file
        let container = RawBytesContainer::from_slice(&packets);
        container.write_to_file(tmp_path).unwrap();

        // Open read-only mmap container
        let mut ro_container = RawBytesContainer::<Packet>::open_mmap_read(tmp_path).unwrap();

        assert!(ro_container.as_slice_mut().is_none());
        assert!(ro_container.as_slice_mut_checked().is_err());
        assert!(ro_container.get_mut(0).is_none());
        assert!(!ro_container.is_mutable());

        // Open read-write mmap container
        let mut rw_container = RawBytesContainer::<Packet>::open_mmap_rw(tmp_path).unwrap();
        rw_container.get_mut_checked(0).unwrap().a = 99;
        rw_container.flush().unwrap();

        let updated = rw_container.as_slice();
        assert_eq!(updated[0].a, 99);
    }
}

fn main() -> Result<(), ContainerError> {
    //use bytemuck::{Pod, Zeroable};
    use tempfile::NamedTempFile;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Pod, Zeroable)]
    struct Packet {
        a: u32,
        b: u16,
        c: u16,
    }

    // Temporary file
    let tmpfile = NamedTempFile::new().expect("Failed to create temp file");
    let tmp_path = tmpfile.path();

    // In-memory container
    let packets = [Packet { a: 1, b: 2, c: 0 }, Packet { a: 4, b: 5, c: 0 }];
    let mut container = RawBytesContainer::from_slice(&packets);

    container.append(&[Packet { a: 7, b: 8, c: 9 }])?;
    container.resize(5, Packet { a: 0, b: 0, c: 0 })?;
    container.write_to_file(tmp_path)?;

    println!(
        "Wrote {} packets to temp file: {:?}",
        container.len(),
        tmp_path
    );

    // Read-only mmap
    let ro_container = RawBytesContainer::<Packet>::open_mmap_read(tmp_path)?;
    println!("Read-only: {:?}", ro_container.as_slice());
    println!("Is mutable? {}", ro_container.is_mutable());

    // Read-write mmap
    let mut rw_container = RawBytesContainer::<Packet>::open_mmap_rw(tmp_path)?;

    // Safe mutable access
    if let Some(slice) = rw_container.as_slice_mut() {
        slice[0].a = 42;
    }

    // Checked mutation
    match rw_container.get_mut(0) {
        Some(packet) => packet.a = 99,
        None => println!("Cannot mutate read-only container"),
    }

    rw_container.flush()?;
    println!(
        "Read-write after modification: {:?}",
        rw_container.as_slice()
    );

    // Temp file automatically cleaned up
    Ok(())
}
