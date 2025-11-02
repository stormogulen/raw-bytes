//
//use bytemuck::{Pod, Zeroable};
use bytemuck_derive::{Pod, Zeroable};
use std::{fs::File, io::Write, path::PathBuf};

use packed_struct_container::PackedStructContainer;

/// A simple packed struct that can be safely converted to/from bytes
#[repr(C)]
#[derive(Clone, Copy, Debug, Zeroable, Pod, PartialEq)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ---------------------------------------------------------
    // Example 1: In-memory container
    // -----------use bytemuck_derive::{Pod, Zeroable};----------------------------------------------
    println!("=== In-memory example ===");

    let points = [
        Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
    ];

    // Create from slice
    let mut container = PackedStructContainer::from_slice(&points);

    println!("Initial len: {}", container.len());
    println!("First element: {:?}", container.get(0));

    // Append another point
    let p = Vec3 {
        x: 2.0,
        y: 2.0,
        z: 2.0,
    };
    container.append(&[p])?;
    println!("After append len: {}", container.len());

    // Iterate over all elements
    for (i, v) in container.iter().enumerate() {
        println!("  [{}] {:?}", i, v);
    }

    // Mutate first element
    if let Some(first) = container.get_mut(0) {
        first.x = 9.9;
    }

    println!("After mutation: {:?}", container.get(0));

    // Save the in-memory data to a binary file
    let path = PathBuf::from("target/test_points.bin");
    std::fs::create_dir_all("target")?;
    let mut file = File::create(&path)?;
    file.write_all(bytemuck::cast_slice(container.as_slice()))?;
    file.flush()?;
    println!("\nSaved {} points to {:?}", container.len(), path);

    // ---------------------------------------------------------
    // Example 2: Memory-mapped file
    // ---------------------------------------------------------
    println!("\n=== Memory-mapped example ===");

    let path = PathBuf::from("target/test_points.bin");

    {
        // Create and write with mmap RW
        let mmap_container = PackedStructContainer::<Vec3>::open_mmap_rw(&path)?;
        //
        println!("Opened mmap file: {:?}", path);
        let mmap_ro = PackedStructContainer::<Vec3>::open_mmap_read(&path)?;
        for (i, v) in mmap_ro.iter().enumerate() {
            println!("  [{}] {:?}", i, v);
        }

        mmap_container.flush()?; // Ensure data written to disk
        println!("Wrote and flushed {} elements", mmap_container.len());
    }

    {
        // Read back with mmap read-only
        let mmap_ro = PackedStructContainer::<Vec3>::open_mmap_read(&path)?;
        println!("Read-only mmap len: {}", mmap_ro.len());
        for (i, v) in mmap_ro.iter().enumerate() {
            println!("  [{}] {:?}", i, v);
        }
    }

    Ok(())
}
