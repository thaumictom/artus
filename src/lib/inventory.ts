import { LazyStore } from '@tauri-apps/plugin-store';

export type InventoryItem = {
	name: string;
	slug?: string;
	isCustom?: boolean;
	category?: string;
	quantity: number;
	marketMedian?: number;
	marketMedianUsesOfferFallback?: boolean;
	ducats?: number;
};

export function inventoryNameKey(name: string) {
	return name.trim().replace(/\s+/g, ' ').toLowerCase();
}

export function inventoryMarketSlug(item: InventoryItem) {
	if (!item.slug || item.isCustom) return undefined;
	return item.name.includes('Relic')
		? item.slug.replace(/_(intact|radiant)$/, '')
		: item.slug;
}

let latestSave: Promise<void> = Promise.resolve();

export function trackInventorySave(save: Promise<void>) {
	latestSave = save;
}

export function waitForInventorySave() {
	return latestSave;
}

export function resetInventory() {
	const previousSave = latestSave;
	const reset = previousSave.catch(() => undefined).then(async () => {
		const store = new LazyStore('inventory.json');
		await store.set('items', [] as InventoryItem[]);
		await store.set('newSlugs', [] as string[]);
		await store.save();
	});
	trackInventorySave(reset);
	return reset;
}
