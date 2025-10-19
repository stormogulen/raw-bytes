# mft

> Core data model for Minimal Type Format (MTF).

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/stormogulen/<repo>/actions/workflows/mtf.yml/badge.svg)](https://github.com/stormogulen/mft/actions/workflows/mtf.yml)


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