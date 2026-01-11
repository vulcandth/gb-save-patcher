#![forbid(unsafe_code)]

//! Holistic example “game crate” that plugs into the game-agnostic shells.
//!
//! This crate is intentionally small and not intended for publishing.
//!
//! It mirrors the structure of a real game crate:
//! - `game`: save layout and version detection
//! - `symbols`: per-version symbols (stubbed)
//! - `migrations`: a chain of migration patches
//! - `fixes`: one-off fix patches keyed by `dev_type` (stubbed)
//! - `validation`: optional preflight validation hooks (stubbed)
//! - `patcher`: the main entry points used by CLI/WASM callers

mod fixes;
mod game;
mod migrations;
mod patcher;
mod symbols;
mod validation;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

pub use gb_save_core::{PatchLogLevel, SaveBinary};

pub use game::{get_save_version, MIN_SAVE_SIZE, SAVE_VERSION_ABS_ADDRESS};
pub use patcher::{patch_save_bytes, patch_save_bytes_with_log, PatchSaveOutcome};
pub use symbols::{supported_version_from_u16, symbols_for_version, SupportedSaveVersion};
pub use validation::{validate_before_patching, validate_before_patching_with_log};

/// Convenience wrapper suitable for implementing `gb_save_cli::GameCli::detect_version`.
pub fn detect_version(bytes: &[u8]) -> anyhow::Result<u16> {
    patcher::detect_version_for_cli(bytes)
}

/// Convenience wrapper suitable for implementing `gb_save_cli::GameCli::patch`.
pub fn patch(bytes: Vec<u8>, target_version: u16, dev_type: u8) -> anyhow::Result<Vec<u8>> {
    patcher::patch_save_bytes_for_cli(bytes, target_version, dev_type)
}

/// Convenience wrapper suitable for implementing `gb_save_cli::GameCli::patch_with_log`.
#[must_use]
pub fn patch_with_log(bytes: Vec<u8>, target_version: u16, dev_type: u8) -> gb_save_cli::PatchOutcome {
    patcher::patch_save_bytes_with_log_for_cli(bytes, target_version, dev_type)
}
