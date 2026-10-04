import { LazyStore } from '@tauri-apps/plugin-store';

const store = new LazyStore('alert-completions.json');
let pending: Promise<unknown> = Promise.resolve();

export const alertCompletions = $state({
	completedIds: [] as string[],
	loaded: false,
});

const completedIdSet = $derived(new Set(alertCompletions.completedIds));

export function isAlertCompleted(id: string | undefined): boolean {
	return !!id && completedIdSet.has(id);
}

// Serialize reads and writes so reopening the view cannot race a pending save.
function withStore<T>(action: () => Promise<T>): Promise<T> {
	const result = pending.then(action);
	pending = result.catch(() => {});
	return result;
}

export function loadAlertCompletions(presentIds: string[]): Promise<string[]> {
	return withStore(async () => {
		const saved = await store.get<string[]>('completedIds') ?? [];
		const present = new Set(presentIds);
		// Only API presence matters here; expiry dates must not remove progress.
		const retained = saved.filter((id) => present.has(id));
		if (retained.length !== saved.length) {
			await store.set('completedIds', retained);
			await store.save();
		}
		alertCompletions.completedIds = retained;
		alertCompletions.loaded = true;
		return retained;
	});
}

// Navigation loads progress without pruning; cleanup belongs to entering Alerts.
export function initializeAlertCompletions(): Promise<void> {
	return withStore(async () => {
		if (alertCompletions.loaded) return;
		alertCompletions.completedIds = await store.get<string[]>('completedIds') ?? [];
		alertCompletions.loaded = true;
	});
}

export function setAlertsCompleted(ids: string[], completed: boolean): Promise<string[]> {
	return withStore(async () => {
		const saved = await store.get<string[]>('completedIds') ?? [];
		const selected = new Set(ids);
		const updated = completed
			? [...new Set([...saved, ...ids])]
			: saved.filter((value) => !selected.has(value));
		await store.set('completedIds', updated);
		await store.save();
		alertCompletions.completedIds = updated;
		alertCompletions.loaded = true;
		return updated;
	});
}
