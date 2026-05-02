## artus

Artus is a desktop app built with Tauri v2 + SvelteKit + TypeScript (Vite) and a Rust backend. 

The app is a companion app for the game Warframe. 

Its core feature is the in-depth OCR system that scans the screen to check the prices of items in the game. It runs automatically during the relic rewards screen of Fissure missions, but can also be triggered manually using a hotkey. The overlay window always sits on top of the game window and is clickthrough and semi-transparent. The overlay shows the item name and the median price next to it. 

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
- `settings.json` to persist Settings over app relaunches (using tauri-plugin-store)
- Hotkey system, registering and unregistering depending on Warframe's focus
- Fully working OCR system to capture the Warframe screen
  - Aggressive color filtering using color of Warframe Theme setting
  - Strict dictionary mapping
  - Show median prices next to the item name
  - Two overlay methods:
    - Toggle overlay (press Hotkey to run OCR, show the overlay and press it again to hide)
    - Show overlay for a set time (default 10s) and then hide it automatically
  - Auto-hides the overlay whenever Warframe is unfocused
- Automatic Relic Rewards detection
  - Detects when a Relic Reward screen is shown and automatically runs OCR
  - Detects when the Relic Reward screen is closed and automatically hides the overlay
- Fetch dictionary and tradeable items from custom external API
- Market tab with warframe.market API integration and a search box based on the dictionary

## Notes

- Try working with powershell (pwsh) first. If not available, work with bash to run commands.
- Always add some light comments in the code. Do not over-comment.
- Always check that the app is working using pnpm check and cargo check (inside src-tauri) after your changes.
- Optional Wayland layer-shell build (for Linux only):
	pnpm tauri dev --features wayland-layer-shell
