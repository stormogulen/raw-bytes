use bytemuck::Pod;
use memmap2::{Mmap, MmapMut};

/// Storage variants for RawBytesContainer
#[derive(Debug)]
pub enum Storage<T: Pod> {
    InMemory(Vec<T>),
    MmapRO(Mmap),
    MmapRW(MmapMut),
}
