
// use pak::{PakBuilder, PakReader, AssetEntry, AssetType};

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     println!("PAK File Format Example");
    
//     // Create a PAK file
//     println!("\n=== Building PAK ===");
//     let mut builder = PakBuilder::new();
//     builder.compression_level(3);
    
//     // Add some test assets
//     builder.add_asset(AssetEntry::new(
//         "test.txt",
//         b"Hello, PAK!".to_vec(),
//         AssetType::Data
//     ));
    
//     builder.add_asset(AssetEntry::new(
//         "data.bin",
//         vec![1, 2, 3, 4, 5],
//         AssetType::Data
//     ));
    
//     // Build the PAK file
//     // builder.build("test.pak")?;
//     // println!("Built test.pak with {} assets", builder.assets.len());
    
//     // Read the PAK file
//     // println!("\n=== Reading PAK ===");
//     // let pak = PakReader::open("test.pak")?;
//     // let assets = pak.list_assets();
//     // println!("Found {} assets:", assets.len());
//     // for name in assets {
//     //     println!("  - {}", name);
//     // }
    
//     //println!("\nNote: Builder and Reader not yet implemented!");
    
//     Ok(())
// }



use pak::{PakBuilder, PakReader, AssetEntry, AssetType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PAK File Format Demo ===\n");
    
    // Create a test PAK file
    println!("📦 Building PAK file...");
    let mut builder = PakBuilder::new();
    
    // Configure compression
    builder
        .compression_level(3)
        .compress_threshold(512);
    
    // Add various assets
    builder.add_asset(AssetEntry::new(
        "readme.txt",
        b"Welcome to the PAK format!\nThis is a simple archive system.".to_vec(),
        AssetType::Data
    ));
    
    builder.add_asset(AssetEntry::new(
        "config.json",
        br#"{"version": 1, "name": "test_game"}"#.to_vec(),
        AssetType::Data
    ));
    
    // Add a small sprite (simulated)
    let sprite_data: Vec<u8> = (0..64).map(|i| (i * 4) as u8).collect();
    builder.add_asset(AssetEntry::new(
        "player.sprite",
        sprite_data,
        AssetType::Texture
    ));
    
    // Add a large compressible asset
    let large_data = vec![42u8; 2048]; // Highly compressible
    builder.add_asset(AssetEntry::new(
        "level_data.bin",
        large_data,
        AssetType::Data
    ));
    
    // Build the PAK
    let pak_path = "demo.pak";
    builder.build(pak_path)?;
    
    println!("✓ Built {} with {} assets\n", pak_path, builder.asset_count());
    
    // Read the PAK file
    println!("📖 Reading PAK file...");
    let reader = PakReader::open(pak_path)?;
    
    println!("✓ Opened {} (memory-mapped)", pak_path);
    println!("  Total assets: {}\n", reader.asset_count());
    
    // List all assets
    println!("📋 Asset listing:");
    for name in reader.list_assets() {
        if let Some(info) = reader.get_info(&name) {
            let compressed_str = if info.is_compressed {
                format!("→ {} bytes", info.compressed_size)
            } else {
                "uncompressed".to_string()
            };
            
            println!("  • {} ({} bytes, {})", 
                     name, 
                     info.size,
                     compressed_str);
        }
    }
    println!();
    
    // Read specific assets
    println!("📄 Reading specific assets:");
    
    // Read readme
    let readme = reader.get_asset("readme.txt")?;
    let readme_text = String::from_utf8_lossy(&readme);
    println!("\n  readme.txt:");
    for line in readme_text.lines() {
        println!("    {}", line);
    }
    
    // Read config
    let config = reader.get_asset("config.json")?;
    let config_text = String::from_utf8_lossy(&config);
    println!("\n  config.json:");
    println!("    {}", config_text);
    
    // Zero-copy access
    println!("\n🚀 Zero-copy access test:");
    if let Some(slice) = reader.get_asset_slice("player.sprite")? {
        println!("  Got player.sprite as zero-copy slice!");
        println!("  First 8 bytes: {:?}", &slice[..8.min(slice.len())]);
    }
    
    // Memory usage info
    println!("\n💾 Memory efficiency:");
    let file_size = std::fs::metadata(pak_path)?.len();
    println!("  PAK file size: {} bytes", file_size);
    println!("  Memory-mapped: Assets loaded on-demand");
    println!("  Zero-copy: Uncompressed assets use no extra memory");
    
    // Compression stats
    println!("\n📊 Compression stats:");
    let mut total_uncompressed = 0u64;
    let mut total_compressed = 0u64;
    let mut compressed_count = 0;
    
    for name in reader.list_assets() {
        if let Some(info) = reader.get_info(&name) {
            total_uncompressed += info.size;
            if info.is_compressed {
                total_compressed += info.compressed_size;
                compressed_count += 1;
            } else {
                total_compressed += info.size;
            }
        }
    }
    
    let ratio = (total_compressed as f64 / total_uncompressed as f64) * 100.0;
    println!("  Compressed assets: {}/{}", compressed_count, reader.asset_count());
    println!("  Total uncompressed: {} bytes", total_uncompressed);
    println!("  Total compressed: {} bytes", total_compressed);
    println!("  Compression ratio: {:.1}%", ratio);
    
    // Cleanup
    println!("\n🧹 Cleaning up...");
    std::fs::remove_file(pak_path)?;
    println!("✓ Removed {}", pak_path);
    
    println!("\n✨ Demo complete!");
    
    Ok(())
}