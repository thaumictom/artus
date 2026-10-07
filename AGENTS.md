# Artus

Early-alpha Warframe desktop companion for Windows and Linux. Tauri 2/Rust 2021 hosts a Svelte 5 (runes), SvelteKit, TypeScript, Vite, Tailwind 4 and bits-ui frontend. SvelteKit uses adapter-static with SSR disabled; this is a desktop SPA, with no Node server.

## Features and code map

- `src/routes/artus/+page.svelte`: main `artus` window, tab navigation and update prompt. Developer settings display OCR inspection images/text; Market searches items and shows details, buy/sell orders and price/volume charts; Settings controls hotkeys, OCR, overlay and relic reward detection.
- `src/routes/overlay/+page.svelte`: separate transparent, clickthrough `overlay` window. Positions item labels over the game with platinum prices, ducats, volume, vault status, mod colors and salvage/sell thresholds. Supports timed and toggle display.
- `src-tauri/src/ocr/`: capture, preprocessing, Tesseract recognition, word grouping, dictionary matching and price enrichment. `mod.rs` defines payloads/constants; `assets/` contains theme colors and checkmark templates; `build.rs` embeds the bundled `tessdata/eng.traineddata` (supports environment overrides).
- `src-tauri/src/desktop/`: focus-sensitive hotkeys, window tracking/sizing, tray integration and optional Wayland layer-shell support. Default capture is Ctrl+Home; inventory capture is Ctrl+Shift+Home.
- `src-tauri/src/relics/`: DBWIN and visual reward detection, capture coordination and automatic inventory additions. Platform-specific modules retain their Windows/Linux gates.
- `src-tauri/src/inventory/`: authoritative inventory mutations and persistence in `inventory.json`; `domain.rs` keeps mutation rules independent of Tauri. Main and overlay windows read revisioned snapshots and request operations through commands, synchronized by `inventory_changed`. `src/routes/artus/inventory/` provides editing and platinum/ducat totals.
- `src/routes/artus/mastery/`: **work in progress**, not functional mastery tracking. `scripts/generate-mastery-items.mjs` derives `src/lib/data/masterable-items.json` from `@wfcd/items`.
- `src-tauri/src/data/`: central catalog refresh in `api.rs`, world state, Oracle bounties and wiki offerings. `src-tauri/src/market/` provides cached lookups, live orders/statistics, account operations and notification sockets. `src/lib/schemas.ts` validates frontend API responses with Zod.
- `main.rs` registers commands/plugins; `app/` contains startup, updates and developer tooling. Shared `state.rs`, `error.rs` and `store_ext.rs` remain at the root.

## Working conventions

- Use `$lib/components/WarframeItem.svelte` for desktop item names, metadata, mastery/ownership/listing indicators, and market navigation. Pass a slug or game reference; its shared window context reads cached metadata and relationships once rather than fetching per rendered item. Prefer the Thaumictom game catalog's name for ordinary items. For `Components`, use the complete cached `/wfm-items` name (also supplied by `/tradeable-items`), falling back to the supplied display name before the catalog's short recipe label; do not construct component names by joining parent records. Use the shared metadata subtext (type/category, mastery requirement, compatibility) on every page; keep listing rank/subtype and other view-specific information separate. Keep view-specific controls and overlay geometry/focus behavior outside the component.
- Follow existing Svelte runes, TypeScript and Rust patterns; reuse `$lib/components` and theme tokens in `src/app.css`. Add brief comments for non-obvious behavior.
- Before adding UI controls, check `$lib/components` and existing usages. Reuse or extend shared components instead of styling one-off controls in views; use `RadioGroup.svelte` with `variant="tabs"` for tab selectors.
- Keep Tauri command names/arguments and Rust event payloads aligned with frontend consumers. OCR events are `ocr_processing`, `ocr_result`, `ocr_clear`, `ocr_developer_image` and `ocr_text_result`; inventory additions require `is_inventory_add` and a matched slug. Clean up event listeners on unmount.
- Settings live in `settings.json`: frontend state/defaults in `src/lib/settings.svelte.ts`, backend access through `store_ext.rs::SettingsExt`. Check both sides when changing keys or defaults; frontend and Rust fallback values are not all identical. Separate windows do not share Svelte state.
- Preserve optional price fields, moving-average/current-offer distinctions and relic refinement fallback flags. Missing prices must not silently become real zero prices.
- Preserve overlay transparency, clickthrough, non-activation, geometry/DPI conversion, focus handling and sequence-based timer cancellation. Keep Windows-specific code and Linux Wayland feature gates intact. EE.log remains read-only; do not modify game files or inject into the game.
- Register new commands in `main.rs`; check window labels and `src-tauri/capabilities/` when changing desktop permissions. Regenerate mastery data instead of hand-editing it; do not edit build output or generated Tauri/Svelte files.
- Do not add migration code, compatibility shims, or automatic cleanup for old settings, stored data, or application behavior. Assume this app has a single user; update current defaults and code directly.
- Do not create regression tests. Use the existing validation commands and focused manual checks when verification is needed.
- Route new outbound API endpoints through one backend fetch path and give each catalog a shared cache. Keep API response caches in memory for the current session; persist only the ETag-backed Thaumictom `/items` catalog and its validator. Read cached item metadata from commands instead of fetching once per item or per window. Refresh the Thaumictom catalogs and warframe.market `/v2/items` on the central 30-minute cycle and through Maintenance; keep `/items` conditional on its ETag. Use the shared `Artus/<version> (+https://github.com/thaumictom/artus)` User-Agent for HTTP and WebSocket connections. Avoid unbounded per-item request loops against warframe.market.
- `/wfm-items` supplies market item IDs and listing variant fields; use its cached slug/ID indexes for identity lookups, listing creation, and searches. Reserve cached warframe.market `/v2/items` for Market view details such as localized text and images.
- Use the centrally refreshed, cached `https://api.thaumictom.de/warframe/v2/items` catalog for game item metadata and classification. Its keys are `uniqueName`; join world state inventory entries on that key and read the catalog through `get_cached_market_items`. Do not plan or implement item metadata features using `@wfcd/items` or generated npm-package snapshots: catalog updates must reach the app without requiring an app release. For Baro filters, classify items as Mods or Weapons only when their catalog metadata matches; everything else, including unmatched items and appearance items, belongs to Misc.

## Commands and validation

- Use pnpm; prefer PowerShell, with Bash as a fallback. Install: `pnpm install`.
- Full desktop development: `pnpm tauri dev`; `pnpm dev` runs only Vite on port 1420 and cannot validate native integrations.
- After code changes: `pnpm check` and `cargo check --manifest-path src-tauri/Cargo.toml`. Report existing failures separately. No automated test suite is currently configured; manually exercise affected desktop flows, especially capture, focus changes, persistence and overlay dismissal.
- Frontend build: `pnpm build`; packaged desktop build: `pnpm tauri build`. Native prerequisites are described in README and `.github/workflows/build-tauri.yml`.
- Linux layer-shell: `pnpm tauri dev --features wayland-layer-shell` (requires `gtk-layer-shell` development files).
- Regenerate mastery data: `pnpm generate:mastery-items`.
- Version changes: `pnpm update-version <version>`, then `pnpm check:version-sync`. Keep `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` synchronized; the pre-commit hook checks this. GitHub Actions builds Windows NSIS/Linux AppImage releases when the version tag has no existing release.
