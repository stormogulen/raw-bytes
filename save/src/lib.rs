//! save-system: A small Merkle-verified save file library
//!
//! Supports integrity-checked serialization of containers.
//! Built for use with `PackedStructContainer.
pub mod merkle;
pub mod save;

pub use save::{save_game, load_game};
