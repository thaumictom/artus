import { inventoryStore, waitForInventorySave, type InventoryItem } from '$lib/inventory';

export const inventory = $state({
	items: [] as InventoryItem[],
	newSlugs: [] as string[],
	ready: false,
	error: '',
});

let generation = 0;
let itemsRevision = 0;
let slugsRevision = 0;
let readRequest = 0;

export async function reloadInventory() {
	const current = generation;
	const request = ++readRequest;
	try {
		await waitForInventorySave().catch(() => undefined);
		const itemsAtRead = itemsRevision;
		const slugsAtRead = slugsRevision;
		const [items, newSlugs] = await Promise.all([
			inventoryStore.get<InventoryItem[]>('items'),
			inventoryStore.get<string[]>('newSlugs'),
		]);
		if (current !== generation || request !== readRequest) return;
		// A store event arriving during these reads carries the newer value.
		if (itemsAtRead === itemsRevision) inventory.items = items ?? [];
		if (slugsAtRead === slugsRevision) inventory.newSlugs = newSlugs ?? [];
		inventory.ready = true;
		inventory.error = '';
	} catch (error) {
		if (current !== generation || request !== readRequest) return;
		inventory.error = 'Could not load inventory.';
		console.error('Could not load shared inventory:', error);
	}
}

/** Owned by the window's shared item context, independent of the active tab. */
export function initializeInventory() {
	const current = ++generation;
	let unlisten: (() => void) | undefined;
	void inventoryStore.onChange<unknown>((key, value) => {
		if (current !== generation) return;
		if (key === 'items' && Array.isArray(value)) {
			itemsRevision++;
			inventory.items = value as InventoryItem[];
		}
		if (key === 'newSlugs' && Array.isArray(value)) {
			slugsRevision++;
			inventory.newSlugs = value as string[];
		}
	}).then((stop) => {
		if (current !== generation) stop();
		else {
			unlisten = stop;
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
