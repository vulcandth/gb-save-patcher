---
post_title: "Web Log CSS Contract"
author1: "GitHub Copilot"
post_slug: "web-log-css-contract"
microsoft_alias: "n/a"
featured_image: "https://example.com/placeholder.jpg"
categories:
  - web
  - wasm
  - rust
  - tooling
tags:
  - logging
  - css
  - api
  - wasm
ai_note: "AI-assisted: documented the existing JS log/outcome shape and recommended CSS classes."
summary: "Defines the stable CSS class names emitted by gb-save-web for PatchLogEntry logs, so UIs can render colored logs consistently across games."
post_date: "2026-01-10"
---

## Overview

`gb-save-web` exposes helpers that convert Rust `PatchLogEntry` values into a JS-friendly representation.
Each log entry includes:

- `level`: stable severity string (`"info" | "warn" | "error"`)
- `className`: stable CSS class string for drop-in styling
- `source`: a stable log source identifier (e.g., patch id)
- `message`: human-readable log message

The contract is implemented in `gb_save_web::logs_to_js(...)` and used by `gb_save_web::patch_outcome_to_js(...)`.

## Log entry shape

A UI should treat this shape as stable:

```ts
type PatchLogLevel = "info" | "warn" | "error";

type PatchLogEntry = {
  level: PatchLogLevel;
  className: string;
  source: string;
  message: string;
};
```

## CSS classes

`className` is a space-separated class list with:

- a base class: `gb-save-log`
- a severity modifier:
  - `gb-save-log--info`
  - `gb-save-log--warn`
  - `gb-save-log--error`

Recommended usage:

- If you render logs as DOM elements: set `element.className = entry.className`.
- If you render logs as text only: you can ignore `className` and style by `level`.

## Recommended default styling

A minimal contract-friendly CSS snippet:

```css
.gb-save-log {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  white-space: pre-wrap;
}

.gb-save-log--info {
  color: #9aa4b2;
}

.gb-save-log--warn {
  color: #d8a107;
}

.gb-save-log--error {
  color: #e05d5d;
}
```

## Notes for UI implementers

- Do not parse `message` to infer severity.
- Prefer `level` for logic and `className` for styling.
- `className` may gain additional classes in the future (e.g., for game-specific sources). UIs should ignore unknown classes rather than depending on exact equality.
