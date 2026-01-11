---
post_title: "gb-save-web: www UI shell"
author1: "matth"
post_slug: "gb-save-web-www"
microsoft_alias: "n/a"
featured_image: "https://placehold.co/1200x630/png?text=gb-save-web%20www"
categories:
	- web
	- wasm
	- rust
	- tooling
tags:
	- ui
	- static-site
	- wasm
	- integration
ai_note: "AI-assisted: updated to be publish-ready and match the current downstream-configurable UI design."
summary: "Player-facing static web UI shell that downstream game crates can configure or replace to patch saves locally in the browser."
post_date: "2026-01-10"
---

## What this folder is

This folder is a **player-facing** static web UI shell.

- It does **not** contain game logic.
- Game-specific instructions, target versions, and GitHub links are configured by the downstream game crate.

## Downstream customization

Downstream game crates can customize in three ways:

1. Replace `config.js`
	- Set the title/subtitle, player instructions, GitHub link, accepted extensions, target version labels, and WASM module path.

2. Replace `styles.css`
	- The UI is driven by CSS variables.
	- You can fully restyle it, or just tweak variables from `config.js`.

3. Replace the entire `www/` folder
	- If you want a bespoke UI, copy this folder into your game repo and change whatever you want.

## Expected WASM exports

The UI expects the downstream WASM module to export:

- `get_save_version(bytes) -> number`
- `patch_save(bytes, target_version, dev_type) -> Uint8Array`
- `patch_save_with_log(bytes, target_version, dev_type) -> { bytes, logs, error }`

The `patch_save_with_log` return shape should match the canonical JS object produced by `gb-save-web`.

