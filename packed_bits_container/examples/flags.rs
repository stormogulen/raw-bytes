use packed_bits_container::flags::FlagsContainer;

const FLAG0: u32 = 1 << 0;
const FLAG1: u32 = 1 << 1;
const FLAG2: u32 = 1 << 2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fc = FlagsContainer::<3>::new_in_memory();

    // Push some flags
    fc.push(FLAG0 | FLAG2)?;
    fc.push(FLAG1)?;

    println!("Initial container:");
    for (i, flags) in fc.iter().enumerate() {
        println!("Element {}: {:03b}", i, flags);
    }

    // Set FLAG2 on element 1
    fc.set_mask(1, FLAG2)?;
    // Clear FLAG2 on element 0
    fc.clear_mask(0, FLAG2)?;

    println!("\nAfter modifying flags:");
    for (i, flags) in fc.iter().enumerate() {
        print!("Element {}: {:03b} -> flags set: ", i, flags);
        if let Some(iter) = fc.iter_flags(i) {
            for f in iter {
                print!("{} ", f);
            }
        }
        println!();
    }

    Ok(())
}
