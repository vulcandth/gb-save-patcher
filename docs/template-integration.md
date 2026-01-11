---
post_title: "Template Integration: Adding a New Game Crate"
author1: "GitHub Copilot"
post_slug: "template-integration"
microsoft_alias: "n/a"
featured_image: "https://example.com/placeholder.jpg"
categories:
  - rust
  - cli
  - web
  - wasm
  - tooling
tags:
  - architecture
  - crates
  - logging
  - integration
ai_note: "AI-assisted: drafted integration guidance based on existing shells and example crate."
summary: "How to integrate a new game-specific crate with the game-agnostic gb-save-cli and gb-save-web shells (CLI + WASM) while keeping gb-save-core reusable."
post_date: "2026-01-10"
---

## Goal

This repo treats `gb-save-core`, `gb-save-cli`, and `gb-save-web` as game-agnostic “shell” crates.
A game-specific crate (for one particular game / ROM hack) provides:

- the actual patching logic (version detection, migrations, fix patches)
- a CLI entrypoint by implementing the CLI shell trait
- optional `wasm-bindgen` exports for web use

For a working reference implementation, see the holistic template crate under `examples/gb-save-game-template/`.

## Minimal crate shape

A game crate typically needs:

- `src/lib.rs`: game-specific patching logic and a `patch_with_log(...) -> gb_save_cli::PatchOutcome` helper
- `src/main.rs`: a small CLI binary that implements `gb_save_cli::GameCli`
- `src/wasm.rs` (optional): `wasm-bindgen` exports that use `gb-save-web` helpers

If you want the crate to be a non-published template, set `publish = false`.

If your game crate lives in a separate repository (recommended), prefer a git dependency instead of a relative path dependency.

```toml
[package]
name = "gb-save-your-game"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
anyhow = "1"
gb-save-core = { path = "../../crates/gb-save-core" }
gb-save-cli = { path = "../../crates/gb-save-cli" }
gb-save-web = { path = "../../crates/gb-save-web" }

# Alternatively, for a separate repository:
# gb-save-core = { git = "https://github.com/matth/gb-save-patcher", package = "gb-save-core" }
# gb-save-cli  = { git = "https://github.com/matth/gb-save-patcher", package = "gb-save-cli" }
# gb-save-web  = { git = "https://github.com/matth/gb-save-patcher", package = "gb-save-web" }

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
```

## CLI integration (`gb-save-cli`)

Implement the `gb_save_cli::GameCli` trait in your binary. The shell takes care of argument parsing, output formatting, log printing, JSON output, and color.

```rust
use anyhow::Result;

struct YourGameCli;

impl gb_save_cli::GameCli for YourGameCli {
    fn detect_version(bytes: &[u8]) -> Result<u16> {
        gb_save_your_game::detect_version(bytes)
    }

    fn patch(bytes: Vec<u8>, target: u16, dev_type: u8) -> Result<Vec<u8>> {
        let outcome = gb_save_your_game::patch_with_log(bytes, target, dev_type);
        match (outcome.bytes, outcome.error) {
            (Some(bytes), None) => Ok(bytes),
            (None, Some(err)) => anyhow::bail!(err),
            _ => anyhow::bail!("unexpected patch outcome"),
        }
    }

    fn patch_with_log(bytes: Vec<u8>, target: u16, dev_type: u8) -> gb_save_cli::PatchOutcome {
        gb_save_your_game::patch_with_log(bytes, target, dev_type)
    }
}

fn main() -> Result<()> {
    gb_save_cli::run::<YourGameCli>()
}
```

### Logging expectations

Return logs as structured `gb_save_core::PatchLogEntry` values. The CLI decides what to print based on `--quiet`, `-v/-vv`, and `--format`.

Common patterns:

- Warnings for recoverable issues (e.g., “checksum mismatch, continuing”)
- Errors for abort conditions (e.g., “invalid setting: aborting”)
- Info for verbose diagnostics (e.g., chosen patch id chain / migration plan)

## Web/WASM integration (`gb-save-web`)

`gb-save-web` stays game-agnostic, but provides helpers to convert structured logs and outcomes into a JS-friendly object.

In your game crate, expose a `wasm-bindgen` function that:

1. calls game patching and produces `PatchOutcome`
2. uses `gb_save_web::patch_outcome_to_js(...)` to return a JS object

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn patch_save_with_log(bytes: &[u8], target_version: u16, dev_type: u8) -> JsValue {
    let outcome = gb_save_your_game::patch_with_log(bytes.to_vec(), target_version, dev_type);

    gb_save_web::patch_outcome_to_js(
        outcome.bytes.as_deref(),
        &outcome.logs,
        outcome.error.as_deref(),
    )
}
```

If you’re building a custom UI, the stable log styling contract is documented in `docs/web-log-css-contract.md`.

## Keeping `gb-save-core` reusable

Use `gb-save-core` for:

- safe buffer reading/writing and typed addresses
- symbol database parsing / resolution utilities
- patch framework structures (logs, planning, error model)

Keep game-specific layout, checksums, and offset rules inside the game crate.

## Building and running

### CLI

```powershell
cargo run --manifest-path .\examples\gb-save-game-template\Cargo.toml -- version path\to\save.sav
cargo run --manifest-path .\examples\gb-save-game-template\Cargo.toml -- patch --in in.sav --out out.sav --target 2 --format human -v
cargo run --manifest-path .\examples\gb-save-game-template\Cargo.toml -- patch --in in.sav --out out.sav --target 2 --format json
```

### WASM

```powershell
cargo build --manifest-path .\examples\gb-save-game-template\Cargo.toml --target wasm32-unknown-unknown
```
