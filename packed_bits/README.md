# packed_bits

> Bit-level access utilities for working with binary structures.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A low-level library for extracting and setting fields in bit-packed formats.

---

## Features
- Bit slicing and shifting utilities.
- Type-safe read/write of bit ranges.
- No unsafe code by default.

---

## Example

```rust
use packed_bits::BitField;

let mut data = 0b10110000u8;
let bit = BitField::new(3, 1).get(data);
println!("Bit 3 = {}", bit);
```

Back to [Workspace Overview](../README.md)