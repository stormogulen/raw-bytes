# raw-bytes

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
    let mut container = RawBytesContainer::<Packet>::new_in_memory();
    container.push(Packet { id: 1, value: 42.0 })?;
    container.push(Packet { id: 2, value: 99.9 })?;

    for packet in container.as_slice() {
        println!("Packet {} => {}", packet.id, packet.value);
    }

    Ok(())
}
```

---

## License

MIT
