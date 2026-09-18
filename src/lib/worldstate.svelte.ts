import { invoke } from '@tauri-apps/api/core';
import { WorldStateSchema, WorldCatalogSchema, type WorldState, type WorldCatalog } from './worldstate';

// Module state survives Dashboard unmounts when switching tabs. No network timers.
export const dashboard = $state({
	world: null as WorldState | null,
	catalog: {} as WorldCatalog,
	fetchedAt: null as number | null,
	loading: false,
	error: null as string | null,
	catalogError: false,
});
let started = false;
let pending: Promise<void> | null = null;

export function reloadWorldState(): Promise<void> {
	if (pending) return pending;
	dashboard.loading = true;
	dashboard.error = null;
	pending = (async () => {
		try {
			const world = WorldStateSchema.parse(await invoke('get_world_state'));
			dashboard.world = world;
			dashboard.fetchedAt = Date.now();
		} catch (error) {
			console.error('Could not load world state:', error);
			dashboard.error = 'Could not fetch world state. Check your connection and try Reload.';
		} finally {
			dashboard.loading = false;
			pending = null;
		}
	})();
	return pending;
}

export function initializeWorldState() {
	if (started) return;
	started = true;
	void reloadWorldState();
	// Reuse the existing startup disk cache, without another catalog download.
	void invoke('get_cached_market_items').then((value) => {
		dashboard.catalog = WorldCatalogSchema.parse(value);
	}).catch((error) => {
		console.error('Could not read dashboard item names:', error);
		dashboard.catalogError = true;
	});
}
