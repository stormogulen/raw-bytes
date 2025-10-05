# raw-bytes

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A lightweight Rust library for working with raw, typed byte storage.  
Supports **in-memory vectors** and **memory-mapped files** (both read-only and read/write).

## Features
- Store and access raw bytes as typed `Pod` values.
- Multiple backends:
  - `Vec<T>` (heap in-memory).
  - Read-only memory-mapped files.
  - Read/write memory-mapped files.
- Safe API on top of [`bytemuck`](https://crates.io/crates/bytemuck) and [`memmap2`](https://crates.io/crates/memmap2).

---

## Installation

Add this to your `Cargo.toml`:
```toml
[dependencies]
raw-bytes = "0.1.0"
bytemuck = { version = "1.14", features = ["derive"] }


## Building

```bash
cargo build
```

## Running Tests

```bash
cargo test
```

## Running Examples

The crate comes with examples that demonstrate usage.

```bash
cargo run --example in_memory
cargo run --example mmap_read_only
cargo run --example mmap_read_write
cargo run --example full_workflow
```

## Generating Documentation

Build and open the API documentation in your browser:

```bash
cargo doc --open
```


---

## Example

```rust
use raw_bytes::RawBytesContainer;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Packet {
    id: u32,
    value: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut container = RawBytesContainer::from_slice(&[]);
	container.append(&[Packet { id: 1, value: 42.0 }])?;
	container.append(&[Packet { id: 2, value: 99.9 }])?;

    for packet in container.as_slice() {
        println!("Packet {} => {}", packet.id, packet.value);
    }

    Ok(())
}
```

```rust
use raw_bytes::{RawBytesContainer, ContainerError};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Packet {
    id: u32,
    value: f32,
}

fn main() -> Result<(), ContainerError> {
    // In-memory
    let mut container = RawBytesContainer::from_slice(&[
        Packet { id: 1, value: 42.0 },
        Packet { id: 2, value: 99.9 },
    ]);
    
    // Write to disk
    container.write_to_file("data.bin")?;
    
    // Memory-map for zero-copy access
    let mapped = RawBytesContainer::<Packet>::open_mmap_read("data.bin")?;
    
    for packet in mapped.as_slice() {
        println!("Packet {} => {}", packet.id, packet.value);
    }
    
    Ok(())
}
```

---

## License

MIT
