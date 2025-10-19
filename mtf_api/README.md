# mft_api

> Dynamic runtime API for interacting with MFT data.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Provides reflection, dynamic field access, and runtime mutation of modular field data.

---

## Features
- Dynamic type system for MFT fields.
- Safe mutation and inspection APIs.
- Extensible plugin-style design.

---

## Example

```rust
use mft_api::DynamicContainer;

let mut container = DynamicContainer::new();
container.insert_field("x", 42u32)?;
println!("x = {}", container.get::<u32>("x")?);
````



Back to [Workspace Overview](../README.md)