---
post_title: "Example Game Template (Not Published)"
author1: "GitHub Copilot"
post_slug: "example-game-template"
microsoft_alias: "n/a"
featured_image: "https://example.com/placeholder.jpg"
categories:
  - rust
  - cli
  - wasm
  - web
tags:
  - examples
  - architecture
  - logging
ai_note: "AI-assisted: scaffolded a holistic example crate."
summary: "A holistic, non-published example crate showing how a game crate can plug into gb-save-cli (CLI shell) and gb-save-web (WASM/JS helpers)."
post_date: "2026-01-10"
---

## What this is

- A small, holistic example “game crate” living under `examples/`.
- Not part of the workspace and not intended to be published (`publish = false`).
- Demonstrates both:
  - `gb-save-cli` integration via `GameCli`
  - `wasm-bindgen` exports using `gb-save-web` helpers

This example is structured like a real game crate:

- `src/game.rs`: save layout + version detection
- `src/migrations/`: migration patch chain
- `src/fixes.rs`: fix patches keyed by `dev_type` (stubbed)
- `src/symbols.rs`: per-version symbol loading (stubbed)
- `src/validation.rs`: preflight validation hooks (stubbed)
- `src/patcher.rs`: main patch entry points (`patch_save_bytes*`)

## Getting started (what to edit first)

If you copy this example into a real game crate, these are the files you should update first:

- `src/game.rs`: set the save version address/format (`SAVE_VERSION_ABS_ADDRESS`, `get_save_version`)
- `src/migrations/*`: implement your real migration chain (and delete the placeholder migrations)
- `src/symbols.rs`: load the right symbols per version (or remove symbols if your game doesn’t need them)
- `src/validation.rs`: enforce any “must be true before patching” rules for safety
- `src/fixes.rs`: register any one-off fix patches keyed by `dev_type` (optional)

If you’re integrating with the web demo in `crates/gb-save-web/www`, also update:

- `src/wasm.rs`: export the WASM entry points you want the web UI to call
- `crates/gb-save-web/www/config.js`: set the game title, instructions, target version labels, and GitHub link

## CLI usage

From repo root:

```powershell
cargo run --manifest-path .\examples\gb-save-game-template\Cargo.toml -- version path\to\save.sav
cargo run --manifest-path .\examples\gb-save-game-template\Cargo.toml -- patch --in in.sav --out out.sav --target 2 --format human -v
cargo run --manifest-path .\examples\gb-save-game-template\Cargo.toml -- patch --in in.sav --out out.sav --target 2 --format json
```

## WASM build

```powershell
cargo build --manifest-path .\examples\gb-save-game-template\Cargo.toml --target wasm32-unknown-unknown
```

This example exports:

- `get_save_version(bytes)`
- `patch_save(bytes, target_version, dev_type)`
- `patch_save_with_log(bytes, target_version, dev_type)`

`patch_save_with_log` returns the canonical JS object shape produced by `gb-save-web`.
