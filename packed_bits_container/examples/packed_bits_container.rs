use packed_bits_container::PackedBitsContainer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pb = PackedBitsContainer::<7>::new_in_memory();

    pb.push(5)?;
    pb.push(127)?;

    println!("Length: {}", pb.len());
    println!("Values:");
    for v in pb.iter() {
        println!("{}", v);
    }

    Ok(())
}
