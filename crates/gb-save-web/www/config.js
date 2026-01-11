export const SITE_CONFIG = {
  game: {
    title: "Example Game Save Patcher",
    subtitle: "Patches your save in your browser (no uploads).",

    // Downstream crates should set this to their repo/page.
    githubUrl: null,
    githubLabel: "View source",
  },

  // Downstream crates can override these CSS variables (or replace styles.css entirely).
  theme: {
    "--bg": "#0b1220",
    "--panel": "rgba(255, 255, 255, 0.06)",
    "--text": "#e7edf5",
    "--muted": "rgba(231, 237, 245, 0.7)",
    "--accent": "#7dd3fc",
    "--danger": "#fb7185",
    "--ok": "#86efac",
    "--warn": "#fbbf24",
    "--border": "rgba(231, 237, 245, 0.15)",
    "--btn-bg": "rgba(125, 211, 252, 0.15)",
    "--btn-border": "rgba(125, 211, 252, 0.45)",
    "--btn-bg-hover": "rgba(125, 211, 252, 0.22)",
    "--btn-border-hover": "rgba(125, 211, 252, 0.65)",
    "--link-bg": "rgba(134, 239, 172, 0.14)",
    "--link-border": "rgba(134, 239, 172, 0.45)",
    "--link-bg-hover": "rgba(134, 239, 172, 0.20)",
    "--link-border-hover": "rgba(134, 239, 172, 0.65)",
  },

  wasm: {
    // Downstream crates can point this at their own wasm-pack output.
    // For example: "./pkg/gb_save_game.js"
    modulePath: "./pkg/gb_save_game.js",
  },

  ui: {
    // Default is player-friendly. Downstream crates can enable this if they support fix patches.
    showAdvancedMode: false,
  },

  // Player-facing instructions. Downstream crates should fully replace this list.
  instructions: [
    "Back up your original save file somewhere safe.",
    "Choose your save file below.",
    "Pick a target version, then click Patch.",
    "Download the patched save and load it in your emulator/flashcart.",
  ],

  // Dropdown options for target version. Labels are player-facing.
  // Values must match what the downstream WASM patcher expects.
  targetVersions: [
    { value: 10, label: "Latest" },
  ],

  acceptedExtensions: [".sav", ".srm"],
};
