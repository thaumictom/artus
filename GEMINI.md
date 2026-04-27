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

- Tauri dev server URL: http://localhost:1420
- Optional Wayland layer-shell build (for Linux only):
	pnpm tauri dev --features wayland-layer-shell

#

✦ I have implemented the "Automatic relic reward detection" feature, which monitors the Warframe EE.log file and
  automatically triggers the overlay when rewards are detected, and hides it when the screen is closed.

  Key Changes:
   - Backend State & Persistence: 
       - Added relic_reward_detection to AppState and ensured it is persisted in settings.json.
       - Fixed warframe_log_path to also be persisted in settings.json.
       - Integrated the new relic_rewards module and loaded these settings on application startup.
   - Log Tailing (relic_rewards.rs): 
       - Implemented a performant log tailer that uses BufReader and remembers the last file position to avoid re-reading
         the entire file.
       - Monitors for "ProjectionRewardChoice.lua: Got rewards" to trigger OCR in "Toggle mode" (staying visible).
       - Monitors for "ProjectionRewardChoice.lua: Relic reward screen shut down" to automatically hide the overlay.
   - Frontend Settings: 
       - Created a new RelicRewardDetection.svelte component with a switch to toggle the feature.
       - Added logic to only enable the switch if a valid EE.log path has been configured.
       - Integrated the new setting into the Warframe settings category.
   - OCR Integration: 
       - Exported capture_active_window_with_mode from the OCR module to allow the background tailer to trigger the
         screenshot tool directly.

  Both the log path and the automatic detection toggle will now be carried over when the app restarts.
