# Architecture and state ownership

Audit: 2026-10-07, before the inventory cleanup planned below.

## Current architecture

| Area | Owner / writers | Synchronization and boundary |
| --- | --- | --- |
| Native state | `state.rs::AppState`, with independently locked fields and atomic flags | Tauri commands/events; no global state mutex |
| Settings | Svelte defaults and plugin-store writes; Rust `SettingsExt` reads with call-site defaults; hotkeys/developer tooling also write | Separate window copies; partial subscriptions for themes and overlay settings |
| Inventory | Frontend `inventory.ts` per-window save queues, editable Svelte arrays, overlay store reads, and independent Rust relic additions | Separate store-key events for `items` / `newSlugs`; no transaction across writers |
| Mastery | `mastery.svelte.ts` owns progress, XP, catalog composition and mastered slugs in `mastery.json` | Main-window OCR processing; overlay reads persisted mastered slugs independently |
| Market | Rust owns JWT/session, HTTP, sockets and status operations; Svelte owns session projections and persists the exit-status preference | Auth/status/listing events and commands; overlay fetches its own state |
| Catalogs | Rust catalog/index, OCR dictionary/price, market response, wiki and Oracle caches | Command snapshots and `api_catalogs_fetched`; metadata projections duplicated per window |
| World state | Rust fetches raw documents; Svelte owns parsing, polling, parsed snapshot and notification evaluation | No shared Rust world-state snapshot; domain evaluation depends on main window |
| Notifications | Rust socket transport; Svelte rules, expiry, order filtering/deduplication and history in JSON stores | Transport events feed main-window processing |
| Completion tracking | Svelte-owned alert/calendar/clan completion JSON stores | Window-local reactive copies |
| Overlay/OCR | Rust capture, recognition, focus/process tracking, geometry, clickthrough and cancellation; module-static relic sessions/images | OCR/hotkey events; Svelte selection/dialogs/rendering plus inventory mutation requests |

The `app`, `desktop`, `data`, `market`, `ocr` and `relics` folders still mix domain logic with Tauri handles, plugin stores and delivery. Commands are registered explicitly in `main.rs`. OCR words, market sessions, Oracle/wiki payloads and prices/catalog records have separately maintained Rust and TS/Zod models; several Rust responses are untyped JSON.

## IPC and window boundaries

- Catalog commands (`get_cached_market_items`, `get_cached_wfm_item`, price/dictionary lookups) read Rust caches. `api_catalogs_fetched` prompts each window's shared item context to refresh its presentation indexes. The central native refresh owns network traffic and catalog persistence.
- Market account/listing commands own authentication, status and remote operations. `market_session_changed` and `market_listings_changed` update window projections. `market_order` / `market_socket_state` deliver socket data to frontend notification rules; filtering and durable notification history are still frontend domain work.
- Rust emits `ocr_processing`, `ocr_result`, `ocr_clear`, `ocr_developer_image` and `ocr_text_result`. The overlay maintains its own rendering/interaction session; the main window consumes mastery/developer results. Hotkey, capture, timer and focus effects remain native. Relic sampling state and socket generations also live in module statics rather than managed domain services.
- `get_world_state` fetches and minimally validates a raw document. `worldstate.svelte.ts` applies the third-party parser, constructs missing event/job data and evaluates notifications; main-window timers initiate refreshes. Moving this requires preserving parser behavior rather than merely moving a JSON cache.
- Windows share Rust services and persisted stores, not Svelte module instances. Store-key events and Tauri events currently coexist. Theme/overlay settings and mastery data are independently loaded by the overlay; closing a frontend subscriber must not become a prerequisite for native inventory correctness.

## Locking and ownership

- Keep the existing fine-grained mutexes/atomics. Do not introduce `Mutex<AppState>`.
- The catalog refresh mutex serializes scheduled/manual refresh work; cache locks remain separate. Oracle/wiki async mutexes serialize their own requests. Market status has its own async operation lock; session data is cloned before network work.
- Inventory's per-window queues cannot prevent cross-window/native lost updates. Locking individual plugin-store calls does not serialize a read-modify-save transaction.
- Request counters and store-key revisions protect local ordering, not authoritative ownership. Publish only committed state; version snapshots to tolerate reordered events and command replies.
- Settings defaults differ between frontend and Rust call sites: `overlay_duration_secs` is 25 in `settings.svelte.ts` and falls back to 10 in `ocr/mod.rs`. Move them key by key without silently changing behavior.

## Plan before coding

1. Move all inventory mutations to a dedicated Rust service, with Tauri-free mutation rules and separate persistence/command/event adapters. Use an inventory-specific transaction lock. Keep existing JSON keys, unknown item metadata and unrelated stored entries.
2. Route manual editing, overlay OCR/market changes, reset and relic additions through that service. Both windows consume revisioned snapshots as read-only presentation projections. Remove frontend inventory persistence and full-array writes.
3. Next centralize settings defaults, validation and patches; unify native settings writers and window subscriptions while preserving stored values.
4. Move mastery and notification/completion rules/history into separate services. Move world-state parsing/evaluation after documenting current parser behavior. Keep filters, drafts, selection, animation and layout in Svelte.
5. Replace public `AppState` fields incrementally with explicit services and shared IPC contracts. Keep Tauri/platform code at adapters and preserve overlay focus/cancellation behavior.

The first cleanup adds no migrations, resets, endpoints or broad behavior changes. Validate using existing Rust tests, Cargo checking, frontend type checking and a Tauri build. Native capture/focus/restart flows require separate desktop verification.

## Implemented first cleanup

`inventory::InventoryService` is managed independently from `AppState`. Its own mutex serializes load, mutation and persistence, including native relic additions. `inventory/domain.rs` contains mutation rules without Tauri, window or store dependencies. The adapter keeps the existing `inventory.json` keys (`items`, `newSlugs`), unknown item metadata and unrelated store entries. Reading existing data does not rewrite it. Failed saves leave the committed service state unchanged and restore the plugin's in-memory values.

Commands expose a snapshot, OCR quantity changes, manual additions, row/market quantity changes, new-item dismissal and explicit reset. Successful mutations publish one `inventory_changed` snapshot containing both items and new-item markers. A session-local revision rejects stale events and reads; it is not added to persisted data. No frontend writer can overwrite a complete inventory array from an outdated window copy.

Both windows subscribe before reading their own snapshot. `inventory.svelte.ts` is a read-only presentation projection by convention; the overlay independently derives owned counts. Filters, sort order, dialogs, selection and overlay capture feedback remain local. The overlay's existing session queue still cancels obsolete interaction work; persistence ordering is handled by Rust.

Remaining ownership duplication includes settings/defaults and native settings writers, mastery progress/XP, world-state parsing and notification evaluation/history, and completion tracking. Rust/TypeScript IPC models are still maintained separately, including the new inventory snapshot contract. Catalog/listing/inventory projections still exist per window for rendering; these copies are not authoritative inventory writers. The next cleanup should centralize settings patches/defaults before moving additional domains.

## Validation

- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: all six existing tests passed; no new regression tests added.
- `pnpm check`: zero errors and warnings. `pnpm check:version-sync`: passed.
- `pnpm tauri build --debug --no-bundle -- --target-dir D:/Development/Warframe/artus/src-tauri/target/ownership-check`: passed, including static frontend assets and Windows native linking. The standard target was locked by the running Artus process; using a separate target avoided interrupting it.
- Vite reported dependency warnings (unused polyfill import, `vm-browserify` eval, large chunks). Live native inventory/capture/focus/restart flows and Linux builds were not exercised.
