//! # raw-bytes
//!
//! `raw-bytes` is a lightweight container abstraction for working with
//! plain-old data (POD) types in memory or via memory-mapped files.
//!
//! ## Features
//!
//! - Store data either in memory (`Vec<T>`) or using memory maps (`mmap`).
//! - Ensure type alignment and safety using [`bytemuck`]’s `Pod` guarantee.
//! - Easily switch between read-only and read-write containers.
//! - Support for appending, resizing, flushing, and writing to files.
//!
//! ## Example
//! ```rust
//! use raw_bytes::RawBytesContainer;
//! use bytemuck::{Pod, Zeroable};
//!
//! #[repr(C)]
//! #[derive(Clone, Copy, Debug, Pod, Zeroable)]
//! struct Packet {
//!     a: u32,
//!     b: u16,
//!     c: u16,
//! }
//!
//! let packets = [Packet { a: 1, b: 2, c: 3 }];
//! let container = RawBytesContainer::from_slice(&packets);
//! assert_eq!(container.len(), 1);
//! assert_eq!(container[0].a, 1);
//! ```
//!
//! For more usage patterns, see the examples in the `examples/` directory.
//!
//! [`bytemuck`]: https://docs.rs/bytemuck

pub mod container;
pub mod error;
pub mod storage;

// Re-export core types for convenience

/// Main container type for working with POD data in memory or memory-mapped files.
///
/// See [`RawBytesContainer`](crate::container::RawBytesContainer) for details.
pub use container::RawBytesContainer;

/// Error type used throughout the crate.
///
/// Wraps I/O errors, alignment errors, and unsupported operation errors.
pub use error::ContainerError;

/// Backend storage variants for [`RawBytesContainer`].
///
/// Usually you don't need to use this directly, but it may be useful for inspection.
pub use storage::Storage;
