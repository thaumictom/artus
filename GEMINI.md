## artus

Artus is a desktop app built with Tauri v2 + SvelteKit + TypeScript (Vite) and a Rust backend.

## Stack

- Frontend: Svelte 5 (Runes mode), SvelteKit, Vite, Tailwind CSS, bits-ui
- Backend: Tauri 2 (Rust 2021), serde, tauri plugins (store, updater, opener, global-shortcut)

## Project Structure

- src/: SvelteKit app
- src-tauri/: Tauri app, Rust sources, and config
- static/: static assets
- build/: frontend build output (used by Tauri)

## Scripts

- pnpm dev: run the Vite dev server
- pnpm build: build the frontend
- pnpm preview: preview the frontend build
- pnpm check: run SvelteKit sync + type check
- pnpm check:watch: type check in watch mode
- pnpm tauri dev: run the Tauri app in dev mode

## Current features

- Automatic update checking and download
- `settings.json` to persist Settings over app relaunches
- Hotkey system
- Fetch dictionary and tradeable items from custom external API
- Fully working OCR system
  - Aggressive color filtering using color of Warframe Theme setting
  - Strict dictionary mapping
  - Show median prices next to the item name
  - Two overlay methods:
    - Toggle overlay (press Hotkey to run OCR, show the overlay and press it again to hide)
    - Show overlay for a set time (default 10s) and then hide it automatically
- Market tab with warframe.market API integration and a search box based on the dictionary

## Notes

- Try working with powershell (pwsh) first. If not available, work with bash to run commands.
- Tauri dev server URL: http://localhost:1420
- Optional Wayland layer-shell build (for Linux only):
	pnpm tauri dev --features wayland-layer-shell
- If creating a setting, it needs to be stored locally in settings.json.
