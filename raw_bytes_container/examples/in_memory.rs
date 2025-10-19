//use  bytemuck::{Pod,  Zeroable};
use bytemuck_derive::Pod;
use bytemuck_derive::Zeroable;
use raw_bytes_container::{ContainerError, RawBytesContainer};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Packet {
    a: u32,
    b: u16,
    c: u16,
}

fn main() -> Result<(), ContainerError> {
    let packets = [Packet { a: 1, b: 2, c: 0 }, Packet { a: 4, b: 5, c: 0 }];

    let mut container = RawBytesContainer::from_slice(&packets);
    container.append(&[Packet { a: 7, b: 8, c: 9 }])?;

    println!("In-memory  container:  {:?}", container.as_slice());

    container.resize(5, Packet { a: 0, b: 0, c: 0 })?;
    println!("After  resizing:  {:?}", container.as_slice());

    //  Safe  mutable  access
    if let Some(slice) = container.as_slice_mut() {
        slice[0].a = 42;
    }

    println!("After  mutation:  {:?}", container.as_slice());

    Ok(())
}
