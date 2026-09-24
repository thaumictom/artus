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

export type InventoryOcrWord = {
	slug?: string;
	quantity?: number;
	is_custom?: boolean;
	text: string;
	market_median?: number;
	market_median_from_current_offers?: boolean;
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

export function addOcrWordsToInventory(words: InventoryOcrWord[]) {
	latestSave = latestSave.catch(() => undefined).then(async () => {
		const store = new LazyStore('inventory.json');
		const [savedItems, savedSlugs] = await Promise.all([
			store.get<InventoryItem[]>('items'),
			store.get<string[]>('newSlugs'),
		]);
		const items = savedItems ?? [];
		const newSlugs = Array.isArray(savedSlugs) ? savedSlugs : [];
		let changed = false;
		for (const word of words) {
			if (!word.slug) continue;
			const quantity =
				word.quantity != null && Number.isSafeInteger(word.quantity) && word.quantity > 0
					? word.quantity
					: 1;
			if (!newSlugs.includes(word.slug)) newSlugs.push(word.slug);
			const existing = items.find((item) =>
				item.slug
					? item.slug === word.slug
					: inventoryNameKey(item.name) === inventoryNameKey(word.text),
			);
			if (existing) {
				existing.quantity += quantity;
				existing.slug ??= word.slug;
				existing.isCustom ??= word.is_custom;
				if (word.market_median != null) {
					existing.marketMedian = word.market_median;
					existing.marketMedianUsesOfferFallback = word.market_median_from_current_offers;
				}
				existing.ducats ??= word.ducats;
			} else {
				items.push({
					name: word.text,
					slug: word.slug,
					isCustom: word.is_custom,
					quantity,
					marketMedian: word.market_median,
					marketMedianUsesOfferFallback: word.market_median_from_current_offers,
					ducats: word.ducats,
				});
			}
			changed = true;
		}
		if (!changed) return;
		await store.set('items', items);
		await store.set('newSlugs', newSlugs);
		await store.save();
	});
	return latestSave;
}

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
