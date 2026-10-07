import { getInventorySnapshot, watchInventory, type InventoryItem, type InventorySnapshot } from '$lib/inventory';

// Per-window read-only projection of committed Rust state; UI drafts live in components.
export const inventory = $state({
	items: [] as InventoryItem[],
	newSlugs: [] as string[],
	ready: false,
	error: '',
});
let generation = 0;
let revision = -1;

function applySnapshot(snapshot: InventorySnapshot) {
	if (snapshot.revision < revision) return;
	revision = snapshot.revision;
	inventory.items = snapshot.items;
	inventory.newSlugs = snapshot.newSlugs;
	inventory.ready = true;
	inventory.error = '';
}

export async function reloadInventory() {
	const current = generation;
	try {
		const snapshot = await getInventorySnapshot();
		if (current === generation) applySnapshot(snapshot);
	} catch (error) {
		if (current !== generation) return;
		inventory.error = 'Could not load inventory.';
		console.error('Could not load shared inventory:', error);
	}
}

/** Owned by this window's item context, independent of the active tab. */
export function initializeInventory() {
	const current = ++generation;
	let unlisten: (() => void) | undefined;
	void watchInventory((snapshot) => {
		if (current === generation) applySnapshot(snapshot);
	}).then((stop) => {
		if (current !== generation) stop();
		else {
			unlisten = stop;
			// Subscribe first; the revision protects against an event overtaking this read.
			void reloadInventory();
		}
	}).catch((error) => {
		if (current !== generation) return;
		console.error('Could not observe shared inventory:', error);
		void reloadInventory();
	});
	return () => {
		generation++;
		unlisten?.();
	};
}
