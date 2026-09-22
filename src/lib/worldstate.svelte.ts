import { invoke } from '@tauri-apps/api/core';
import 'reflect-metadata';
import { WorldState, type InitialWorldState } from 'warframe-worldstate-parser';
import { processWorldStateNotifications } from '$lib/notifications.svelte';

export const dashboard = $state({
	world: null as WorldState | null,
	fetchedAt: null as number | null,
	loading: false,
	error: null as string | null,
});
let started = false;
let pending: Promise<void> | null = null;

export function reloadWorldState(): Promise<void> {
	if (pending) return pending;
	dashboard.loading = true;
	dashboard.error = null;
	pending = (async () => {
		try {
			const raw = await invoke<InitialWorldState>('get_world_state');
			const previousWorld = dashboard.world;
			const world = new WorldState(raw, { locale: 'en' });
			dashboard.world = world;
			dashboard.fetchedAt = Date.now();
			await processWorldStateNotifications(world, previousWorld);
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
}
