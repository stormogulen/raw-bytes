# packed_struct_container

> Container for managing serialized or memory-resident packed structs.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Manages multiple packed struct instances, allowing safe zero-copy access and serialization.

---

## Features
- Load/save packed structs from memory or disk.
- Zero-copy access with typed interfaces.
- Compatible with `packed_structs`.

---

## Example

```rust
use packed_struct_container::PackedStructContainer;

let mut container = PackedStructContainer::new::<Header>();
container.append(Header { version: 1, flags: 0, length: 512 })?;
``` 

Back to [Workspace Overview](../README.md)