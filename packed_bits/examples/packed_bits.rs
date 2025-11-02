use packed_bits::PackedBits;

fn main() {
    println!("--- Demonstrating PackedBits<1> ---");
    demonstrate_packed_bits::<1>();

    println!("--- Demonstrating PackedBits<4> ---");
    demonstrate_packed_bits::<4>();

    println!("--- Demonstrating PackedBits<8> ---");
    demonstrate_packed_bits::<8>();

    println!("--- Demonstrating PackedBits<16> ---");
    demonstrate_packed_bits::<16>();

    println!("--- Demonstrating PackedBits<32> ---");
    demonstrate_packed_bits::<32>();

    compare_memory_usage();
}

fn demonstrate_packed_bits<const N: usize>() {
    let mut bits = PackedBits::<N>::new().expect("valid bit width");
    println!("Created PackedBits<{}>.", N);

    for i in 0..10 {
        let value = ((i * 3) as u64 % (1u64 << N)) as u32;
        bits.push(value).expect("push failed");
        println!("Pushed value: {}", value);
    }

    for i in 0..10 {
        if let Some(value) = bits.get(i) {
            println!("Value at index {}: {}", i, value);
        }
    }

    if N > 1 {
        bits.set(2, ((1u64 << (N - 1)) - 1) as u32)
            .expect("set failed");
        println!("Updated index 2 to max value for {} bits.", N);
        println!("Updated value at index 2: {:?}", bits.get(2));
    }

    println!("Iterating through values:");
    for value in bits.iter() {
        print!("{} ", value);
    }
    println!();

    println!(
        "PackedBits<{}>: Stored {} elements in {} bytes.",
        N,
        bits.len(),
        bits.as_bytes().len()
    );
}

fn compare_memory_usage() {
    println!("\n--- Memory Usage Comparison ---");

    let vec: Vec<u32> = (0..10).collect();
    println!(
        "Vec<u32>: Stored {} elements, total memory: {} bytes.",
        vec.len(),
        std::mem::size_of_val(&vec) + vec.capacity() * std::mem::size_of::<u32>()
    );

    let mut packed = PackedBits::<4>::new().expect("valid bit width");
    for i in 0..10 {
        packed.push(i).expect("push failed");
    }
    println!(
        "PackedBits<4>: Stored {} elements, total memory: {} bytes.",
        packed.len(),
        std::mem::size_of_val(&packed) + packed.as_bytes().len()
    );

    println!(
        "Memory savings: {} bytes.",
        (std::mem::size_of_val(&vec) + vec.capacity() * std::mem::size_of::<u32>())
            - (std::mem::size_of_val(&packed) + packed.as_bytes().len())
    );
}
