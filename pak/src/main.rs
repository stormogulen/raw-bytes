
use pak::{PakBuilder, PakReader, AssetEntry, AssetType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("PAK File Format Example");
    
    // Create a PAK file
    println!("\n=== Building PAK ===");
    let mut builder = PakBuilder::new();
    builder.compression_level(3);
    
    // Add some test assets
    builder.add_asset(AssetEntry::new(
        "test.txt",
        b"Hello, PAK!".to_vec(),
        AssetType::Data
    ));
    
    builder.add_asset(AssetEntry::new(
        "data.bin",
        vec![1, 2, 3, 4, 5],
        AssetType::Data
    ));
    
    // Build the PAK file
    // builder.build("test.pak")?;
    // println!("Built test.pak with {} assets", builder.assets.len());
    
    // Read the PAK file
    // println!("\n=== Reading PAK ===");
    // let pak = PakReader::open("test.pak")?;
    // let assets = pak.list_assets();
    // println!("Found {} assets:", assets.len());
    // for name in assets {
    //     println!("  - {}", name);
    // }
    
    //println!("\nNote: Builder and Reader not yet implemented!");
    
    Ok(())
}

