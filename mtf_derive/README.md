# mft_derive

> Derive macros for `mft` field and struct definitions.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This crate provides procedural macros that generate boilerplate code for `mft` types.

---

## Features
- Derive `Field`, `Struct`, and related traits.
- Compile-time validation of field layout and sizes.
- Integrates with the `mft` core crate.

---

## Example

```rust
use mft_derive::MftStruct;

#[derive(MftStruct)]
struct Header {
    id: u16,
    flags: u8,
}
````


Back to [Workspace Overview](../README.md)