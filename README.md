# Tauri + SvelteKit + TypeScript

This template should help get you started developing with Tauri, SvelteKit and TypeScript in Vite.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Wayland Layer-Shell (optional)

To try native Wayland click-through support for the overlay, build `src-tauri` with the `wayland-layer-shell` feature.

1. Install system package `gtk-layer-shell` (must provide `gtk-layer-shell-0.pc` for `pkg-config`).
2. Run Tauri with feature flags:

```bash
pnpm tauri dev --features wayland-layer-shell
```
