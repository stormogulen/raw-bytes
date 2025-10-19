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
    //  Create  in-memory  container
    let packets = [Packet { a: 1, b: 2, c: 0 }, Packet { a: 4, b: 5, c: 0 }];
    let mut container = RawBytesContainer::from_slice(&packets);
    container.append(&[Packet { a: 7, b: 8, c: 9 }])?;

    //  Temporary  file  for  mmap  demonstration
    let temp_file = NamedTempFile::new()?;
    container.write_to_file(temp_file.path())?;

    //  Read-only  mmap
    let ro_container = RawBytesContainer::<Packet>::open_mmap_read(temp_file.path())?;
    println!("Read-only:  {:?}", ro_container.as_slice());

    //  Read-write  mmap
    let mut rw_container = RawBytesContainer::<Packet>::open_mmap_rw(temp_file.path())?;
    if let Some(slice) = rw_container.as_slice_mut() {
        slice[0].a += 100;
    }
    rw_container.flush()?;

    println!(
        "Read-write  after  mutation:  {:?}",
        rw_container.as_slice()
    );
    println!("Container  length:  {}", rw_container.len());
    println!("Is  mutable?  {}", rw_container.is_mutable());
    println!("Capacity  (if  in-memory):  {:?}", rw_container.capacity());

    Ok(())
}
