# packed_bits_container

> Container for managing multiple bit-packed values.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Provides higher-level structures built on top of `packed_bits` for handling lists or arrays of bitfields.

---

## Features
- Batch bitfield access.
- Support for mixed-width fields.
- Integrates with `packed_bits`.

---

## Example

```rust
use packed_bits_container::BitContainer;

let mut container = BitContainer::new(16);
container.set(0, 3, 0b101)?;
println!("{:?}", container);
```

Back to [Workspace Overview](../README.md)