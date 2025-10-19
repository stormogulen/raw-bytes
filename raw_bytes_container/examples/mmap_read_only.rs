//use bytemuck::{Pod, Zeroable};
use bytemuck_derive::Pod;
use bytemuck_derive::Zeroable;
use raw_bytes::{ContainerError, RawBytesContainer};
use tempfile::NamedTempFile;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Packet {
    a: u32,
    b: u16,
    c: u16,
}

fn main() -> Result<(), ContainerError> {
    let packets = [Packet { a: 1, b: 2, c: 0 }];
    let mut container = RawBytesContainer::from_slice(&packets);

    let temp_file = NamedTempFile::new()?;
    container.write_to_file(temp_file.path())?;

    let ro_container = RawBytesContainer::<Packet>::open_mmap_read(temp_file.path())?;
    println!("Read-only container: {:?}", ro_container.as_slice());
    println!("Is mutable? {}", ro_container.is_mutable());

    Ok(())
}
