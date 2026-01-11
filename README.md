---
post_title: "gb-save-patcher (game-agnostic crates)"
author1: "matth"
post_slug: "gb-save-patcher"
microsoft_alias: "n/a"
featured_image: "https://placehold.co/1200x630/png?text=gb-save-patcher"
categories:
  - rust
  - tooling
  - gameboy
  - wasm
  - web
  - cli
tags:
  - save-files
  - patching
  - crates
  - wasm
ai_note: "AI-assisted: documentation was reviewed and updated for the repo split and publish readiness."
summary: "Game-agnostic Rust crates for safe Game Boy save patching, plus reusable CLI and web shells that are implemented by downstream game-specific crates."
post_date: "2026-01-10"
---

## What this repo is

This repository contains **game-agnostic** Rust crates for patching Game Boy-era save files.
It intentionally does **not** ship any game-specific patch logic.

Game-specific patchers live in separate repos and depend on these shared crates.

## Crates

- `gb-save-core`: safe byte access (`SaveBinary`), typed addresses, symbol database support, patch planning, structured logs, and typed errors.
- `gb-save-cli`: a reusable CLI “shell” (argument parsing + output formatting). Downstream game crates implement the `GameCli` trait.
- `gb-save-web`: helpers for WASM/JS glue (log and outcome conversion). The included `www/` folder is a player-facing UI shell.

## Getting started

### Integrate a new game crate

Use the example template:

- `examples/gb-save-game-template/` (non-published reference implementation)

Integration guide:

- `docs/template-integration.md`

### Web demo UI

The player-facing web UI shell lives at:

- `crates/gb-save-web/www/`

Downstream customization is described in that folder.

## Related repositories

- `polished-save-patcher`: an example game-specific implementation that depends on these crates.

## License

MIT (see `LICENSE`).
