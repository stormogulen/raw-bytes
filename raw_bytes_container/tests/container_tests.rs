//use bytemuck::{Pod, Zeroable};
use bytemuck_derive::Pod;
use bytemuck_derive::Zeroable;
use raw_bytes::RawBytesContainer;
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
    //use bytemuck::{Pod, Zeroable};

    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
    struct Packet {
        a: u32,
        b: u16,
        c: u16,
    }

    // Step 1: Start with two packets
    let packets = [Packet { a: 1, b: 2, c: 0 }, Packet { a: 4, b: 5, c: 0 }];
    let mut container = RawBytesContainer::from_slice(&packets);
    assert_eq!(container.len(), 2);
    assert_eq!(container[0].a, 1);

    // Step 2: Mutate the first element
    container.get_mut(0).unwrap().a = 100;
    assert_eq!(container[0].a, 100);

    // Step 3: Append a new packet
    container.append(&[Packet { a: 7, b: 8, c: 0 }]).unwrap();
    assert_eq!(container.len(), 3);

    // Step 4: Resize the container to 5 elements, filling with zeros
    container.resize(5, Packet { a: 0, b: 0, c: 0 }).unwrap();
    assert_eq!(container.len(), 5);

    // Step 5: Increment `a` field of all packets
    if let Some(slice) = container.as_slice_mut() {
        for p in slice.iter_mut() {
            p.a += 1;
        }
    }

    // Step 6: Check expected values
    let expected_a = [101, 5, 8, 1, 1]; // after all modifications
    for (i, p) in container.iter().enumerate() {
        assert_eq!(p.a, expected_a[i], "Mismatch at index {}", i);
    }

    // Step 7: Print packets for visual confirmation (optional)
    for (i, p) in container.iter().enumerate() {
        println!("Packet {}: {:?}", i, p);
    }
}

#[test]
fn test_read_only_and_rw() {
    let packets = [Packet { a: 1, b: 2, c: 0 }];
    let mut container = RawBytesContainer::from_slice(&packets);

    let temp_file = NamedTempFile::new().unwrap();
    container.write_to_file(temp_file.path()).unwrap();

    // Read-only mmap
    let mut ro_container = RawBytesContainer::<Packet>::open_mmap_read(temp_file.path()).unwrap();

    assert!(ro_container.as_slice_mut().is_none());
    assert!(ro_container.as_slice_mut_checked().is_err());
    assert!(ro_container.get_mut(0).is_none());
    assert!(!ro_container.is_mutable());

    // Read-write mmap
    let mut rw_container = RawBytesContainer::<Packet>::open_mmap_rw(temp_file.path()).unwrap();
    rw_container.as_slice_mut().unwrap()[0].a = 42;
    rw_container.flush().unwrap();

    let slice = rw_container.as_slice();
    assert_eq!(slice[0].a, 42);
}
