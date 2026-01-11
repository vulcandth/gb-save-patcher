#![forbid(unsafe_code)]

//! Game Boy save editing primitives.
//!
//! This crate is intentionally game-agnostic: it provides safe helpers for reading and writing
//! bytes/bits in a save buffer, resolving symbol addresses, and orchestrating migrations/fixes via
//! a small patch framework.
//!
//! ## Public API surface
//!
//! The supported public API is what is re-exported from this crate root (for example [`SaveBinary`],
//! [`SymbolDatabase`], and the patch framework types). Internal modules are not considered stable.
//!
//! ## Versioning
//!
//! This crate follows semantic versioning.
//!
//! While the major version is `0`, minor version bumps (`0.x`) may include breaking changes.
//! Patch releases (`0.x.y`) should remain backwards compatible.
//!
//! # Example
//! ```
//! use gb_save_core::{Address, SaveBinary};
//!
//! let mut save = SaveBinary::new(vec![0u8; 16]);
//! save.write_u8(Address(3), 0x42).unwrap();
//! assert_eq!(save.read_u8(Address(3)).unwrap(), 0x42);
//! ```

mod checksum;
mod error;
mod patch_framework;
mod remap;
mod save_binary;
mod symbol_database;
mod types;

pub use checksum::calculate_additive_u16_checksum;
pub use error::{SaveError, SaveResult};
pub use patch_framework::{
    resolve_migration_plan, NoopPatchLogSink, Patch, PatchKind, PatchLogEntry, PatchLogLevel,
    PatchLogSink, PatchMetadata, VecPatchLogSink,
};
pub use remap::{map_bitset, remap_fixed_len_u8_skip_zero, remap_zero_terminated_u8};
pub use save_binary::SaveBinary;
pub use symbol_database::{Symbol, SymbolDatabase};
pub use types::{bits_to_bytes, Address, AddressRange, Size};
