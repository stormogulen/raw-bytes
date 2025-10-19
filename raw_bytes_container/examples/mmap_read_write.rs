//use  bytemuck::{Pod,  Zeroable};
use bytemuck_derive::Pod;
use bytemuck_derive::Zeroable;
use raw_bytes_container::{ContainerError, RawBytesContainer};
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

    let mut rw_container = RawBytesContainer::<Packet>::open_mmap_rw(temp_file.path())?;
    if let Some(slice) = rw_container.as_slice_mut() {
        slice[0].a = 99;
    }
    rw_container.flush()?;

    println!(
        "Read-write  container  after  mutation:  {:?}",
        rw_container.as_slice()
    );

    Ok(())
}
