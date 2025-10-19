# packed_structs

> High-level abstraction for defining structured binary layouts.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Provides macros and helpers for defining and working with bit-packed or byte-packed structs.

---

## Features
- Define structs with packed binary layouts.
- Access fields by name or index.
- Support for both compile-time and runtime introspection.

---

## Example

```rust
use packed_structs::PackedStruct;

#[derive(PackedStruct)]
struct Header {
    version: u8,
    flags: u8,
    length: u16,
}
``` 

Back to [Workspace Overview](../README.md)