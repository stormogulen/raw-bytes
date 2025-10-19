# mft

> Core data model for Modular Field Types (MFT).

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This crate defines the core MFT primitives for representing structured binary data.

---

## Features
- Type-safe MFT field definitions.
- Utilities for alignment, offset, and size handling.
- Basic serialization/deserialization helpers.

---

## Example

```rust
use mft::Field;

let field = Field::new("temperature", 4);
println!("Field: {} ({} bytes)", field.name(), field.size());
```


Back to [Workspace Overview](../README.md)