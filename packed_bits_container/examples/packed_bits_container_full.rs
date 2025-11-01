use packed_bits_container::{container::PackedBitsContainer, flags::FlagsContainer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PackedBitsContainer Demo ===");

    // Create a container with 5 bits per element
    let mut pb = PackedBitsContainer::<5>::new_in_memory();

    // Push some values
    pb.push(3)?;
    pb.push(15)?;
    pb.push(31)?;

    println!("PackedBitsContainer contents:");
    for (i, val) in pb.iter().enumerate() {
        println!("Index {}: {}", i, val);
    }

    // Modify a value
    pb.set(1, 7)?;
    println!("After setting index 1 to 7:");
    for (i, val) in pb.iter().enumerate() {
        println!("Index {}: {}", i, val);
    }

    // Show length and capacity
    println!("Length: {}, Capacity: {}", pb.len(), pb.capacity());

    println!("\n=== FlagsContainer Demo ===");

    const FLAG0: u32 = 1 << 0;
    const FLAG1: u32 = 1 << 1;
    const FLAG2: u32 = 1 << 2;

    let mut fc = FlagsContainer::<3>::new_in_memory();

    // Push some flag masks
    fc.push(FLAG0 | FLAG2)?;
    fc.push(FLAG1)?;

    println!("Initial FlagsContainer contents:");
    for (i, flags) in fc.iter().enumerate() {
        print!("Index {}: {:03b} -> flags set: ", i, flags);
        if let Some(iter) = fc.iter_flags(i) {
            for f in iter {
                print!("{} ", f);
            }
        }
        println!();
    }

    // Modify flags
    fc.set_mask(1, FLAG2)?;
    fc.clear_mask(0, FLAG2)?;
    fc.toggle_mask(0, FLAG1)?;

    println!("\nAfter modifying flags:");
    for (i, flags) in fc.iter().enumerate() {
        print!("Index {}: {:03b} -> flags set: ", i, flags);
        if let Some(iter) = fc.iter_flags(i) {
            for f in iter {
                print!("{} ", f);
            }
        }
        println!();
    }

    Ok(())
}
