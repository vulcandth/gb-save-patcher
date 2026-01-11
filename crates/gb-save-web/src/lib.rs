#![forbid(unsafe_code)]

//! WebAssembly bindings for the save patcher.
//!
//! This crate is intentionally game-agnostic: it provides small helpers that game-specific crates
//! can use when exposing `wasm-bindgen` APIs.
//!
//! ## Public API surface
//!
//! The stable API is [`js`], which contains helpers for converting patch outcomes and logs into
//! JS-friendly shapes.
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
//! # #[cfg(target_arch = "wasm32")]
//! # {
//! use gb_save_core::{PatchLogEntry, PatchLogLevel};
//! use gb_save_web::js::patch_outcome_to_js;
//!
//! let logs = vec![PatchLogEntry {
//!     level: PatchLogLevel::Info,
//!     source: "example",
//!     message: "patched".to_string(),
//! }];
//! let out = patch_outcome_to_js(Some(&[1u8, 2, 3]), &logs, None);
//! drop(out);
//! # }
//! ```

#[cfg(target_arch = "wasm32")]
pub mod js;
