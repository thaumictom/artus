# Artus

Early-alpha Warframe desktop companion for Windows and Linux. Tauri 2/Rust 2021 hosts a Svelte 5 (runes), SvelteKit, TypeScript, Vite, Tailwind 4 and bits-ui frontend. SvelteKit uses adapter-static with SSR disabled; this is a desktop SPA, with no Node server.

## Features and code map

- `src/routes/artus/+page.svelte`: main `artus` window, tab navigation and update prompt. Dashboard displays OCR debug images/text; Market searches items and shows details, buy/sell orders and price/volume charts; Settings controls hotkeys, OCR, overlay and EE.log detection.
- `src/routes/overlay/+page.svelte`: separate transparent, clickthrough `overlay` window. Positions item labels over the game with platinum prices, ducats, volume, vault status, mod colors and salvage/sell thresholds. Supports timed and toggle display.
- `src-tauri/src/ocr/`: capture via xcap, theme/mod color filtering and morphology, Tesseract recognition, word grouping, fuzzy dictionary matching and price enrichment. `mod.rs` defines payloads/constants; `theme_colors.toml` supplies theme colors; `build.rs` embeds the bundled `tessdata/eng.traineddata` (supports environment overrides).
- `hotkeys.rs`, `window_watcher.rs`, `relic_rewards.rs`, `layer_shell.rs`: focus-sensitive shortcuts, game process/window tracking, read-only EE.log tailing for relic reward open/close events, and optional Wayland integration. Default capture is Ctrl+Home; inventory capture is Ctrl+Shift+Home.
- `src/routes/artus/inventory/`: OCR additions, editable quantities and platinum/ducat totals persisted in `inventory.json`. The `ocr_result` listener currently exists only while this tab is mounted.
- `src/routes/artus/mastery/`: **work in progress**, not functional mastery tracking. `scripts/generate-mastery-items.mjs` derives `src/lib/data/masterable-items.json` from `@wfcd/items`.
- `src-tauri/src/market.rs`: warframe.market v2 item/orders requests and v1 statistics. OCR dictionary/prices come from `api.thaumictom.de/warframe/v2/` at startup. `src/lib/schemas.ts` validates frontend API responses with Zod.
- `main.rs` registers commands/plugins; `setup.rs` initializes windows/data/background workers; `state.rs` holds synchronized backend state; `error.rs` provides `AppError`/`AppResult`; `updater.rs` checks releases and installs/relaunches after the UI prompt.

## Working conventions

- Follow existing Svelte runes, TypeScript and Rust patterns; reuse `$lib/components` and theme tokens in `src/app.css`. Add brief comments for non-obvious behavior.
- Keep Tauri command names/arguments and Rust event payloads aligned with frontend consumers. OCR events are `ocr_processing`, `ocr_result`, `ocr_clear`, `ocr_debug_image` and `ocr_text_result`; inventory additions require `is_inventory_add` and a matched slug. Clean up event listeners on unmount.
- Settings live in `settings.json`: frontend state/defaults in `src/lib/settings.svelte.ts`, backend access through `store_ext.rs::SettingsExt`. Check both sides when changing keys or defaults; frontend and Rust fallback values are not all identical. Separate windows do not share Svelte state.
- Preserve optional price fields, moving-average/current-offer distinctions and relic refinement fallback flags. Missing prices must not silently become real zero prices.
- Preserve overlay transparency, clickthrough, non-activation, geometry/DPI conversion, focus handling and sequence-based timer cancellation. Keep Windows-specific code and Linux Wayland feature gates intact. EE.log remains read-only; do not modify game files or inject into the game.
- Register new commands in `main.rs`; check window labels and `src-tauri/capabilities/` when changing desktop permissions. Regenerate mastery data instead of hand-editing it; do not edit build output or generated Tauri/Svelte files.

## Commands and validation

- Use pnpm; prefer PowerShell, with Bash as a fallback. Install: `pnpm install`.
- Full desktop development: `pnpm tauri dev`; `pnpm dev` runs only Vite on port 1420 and cannot validate native integrations.
- After code changes: `pnpm check` and `cargo check --manifest-path src-tauri/Cargo.toml`. Report existing failures separately. No automated test suite is currently configured; manually exercise affected desktop flows, especially capture, focus changes, persistence and overlay dismissal.
- Frontend build: `pnpm build`; packaged desktop build: `pnpm tauri build`. Native prerequisites are described in README and `.github/workflows/build-tauri.yml`.
- Linux layer-shell: `pnpm tauri dev --features wayland-layer-shell` (requires `gtk-layer-shell` development files).
- Regenerate mastery data: `pnpm generate:mastery-items`.
- Version changes: `pnpm update-version <version>`, then `pnpm check:version-sync`. Keep `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` synchronized; the pre-commit hook checks this. GitHub Actions builds Windows NSIS/Linux AppImage releases when the version tag has no existing release.
