# raw-bytes


> A collection of Rust crates for working with **raw, packed, and structured binary data**.


[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build](https://github.com/stormogulen/raw-bytes/actions/workflows/ci.yml/badge.svg)](https://github.com/stormogulen/raw-bytes/actions)

---


## Overview

This workspace contains multiple Rust crates for zero-copy data structures, typed memory containers, and flexible binary manipulation.


| Crate | CI Status | Description |
|-------|------------|-------------|
| **Workspace** | [![Workspace CI](https://github.com/stormogulen/raw-bytes/actions/workflows/ci.yml/badge.svg)](https://github.com/stormogulen/raw-bytes/actions/workflows/ci.yml) | Full workspace build & test |
| **mtf** | [![MFT CI](https://github.com/stormogulen/raw-bytes/actions/workflows/mtf.yml/badge.svg)](https://github.com/stormogulen/raw-bytes/actions/workflows/mtf.yml) | Core metadata format types |
| **mtf_derive** | [![MFT Derive CI](https://github.com/stormogulen/raw-bytes/actions/workflows/mtf_derive.yml/badge.svg)](https://github.com/stormogulen/raw-bytes/actions/workflows/mtf_derive.yml) | Derive macros for MFT |
| **mtf_api** | [![MFT API CI](https://github.com/stormogulen/raw-bytes/actions/workflows/mtf_api.yml/badge.svg)](https://github.com/stormogulen/raw-bytes/actions/workflows/mtf_api.yml) | Public API for interacting with MFT data |
| **packed_bits** | [![Packed Bits CI](https://github.com/stormogulen/raw-bytes/actions/workflows/packed_bits.yml/badge.svg)](https://github.com/stormogulen/raw-bytes/actions/workflows/packed_bits.yml) | Bit-level packing utilities |
| **packed_bits_container** | [![Packed Bits Container CI](https://github.com/stormogulen/raw-bytes/actions/workflows/packed_bits_container.yml/badge.svg)](https://github.com/stormogulen/raw-bytes/actions/workflows/packed_bits_container.yml) | Typed containers for bit-packed data |
| **packed_structs** | [![Packed Structs CI](https://github.com/stormogulen/raw-bytes/actions/workflows/packed_structs.yml/badge.svg)](https://github.com/stormogulen/raw-bytes/actions/workflows/packed_structs.yml) | Struct utilities for working with packed data |
| **packed_struct_container** | [![Packed Struct Container CI](https://github.com/stormogulen/raw-bytes/actions/workflows/packed_struct_container.yml/badge.svg)](https://github.com/stormogulen/raw-bytes/actions/workflows/packed_struct_container.yml) | Containers for packed struct types |
| **raw_bytes_container** | [![Raw Bytes Container CI](https://github.com/stormogulen/raw-bytes/actions/workflows/raw_bytes_container.yml/badge.svg)](https://github.com/stormogulen/raw-bytes/actions/workflows/raw_bytes_container.yml) | Generic raw byte container for POD types |

---

## Getting Started

#### Build all crates
```bash
cargo build --workspace
````

#### Run all tests
```bash 
cargo test --workspace
```

#### Generate docs
```bash
cargo doc --open --workspace
```