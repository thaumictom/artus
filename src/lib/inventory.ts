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

export function changeOcrItemQuantities(changes: { word: InventoryOcrWord; delta: number }[]) {
	const save = latestSave.catch(() => undefined).then(async () => {
		const applied = new Map<string, number>();
		if (!changes.some(({ word, delta }) => word.slug && Number.isSafeInteger(delta) && delta !== 0)) {
			return applied;
		}
		const store = new LazyStore('inventory.json');
		const [savedItems, savedSlugs] = await Promise.all([
			store.get<InventoryItem[]>('items'),
			store.get<string[]>('newSlugs'),
		]);
		const items = savedItems ?? [];
		const newSlugs = Array.isArray(savedSlugs) ? savedSlugs : [];
		for (const { word, delta } of changes) {
			if (!word.slug || !Number.isSafeInteger(delta) || delta === 0) continue;
			const existing = items.find((item) =>
				item.slug
					? item.slug === word.slug
					: inventoryNameKey(item.name) === inventoryNameKey(word.text),
			);
			const previous = existing?.quantity ?? 0;
			const next = Math.max(0, previous + delta);
			const actual = next - previous;
			if (actual === 0) continue;
			if (existing) {
				if (next === 0) {
					items.splice(items.indexOf(existing), 1);
					const newSlugIndex = newSlugs.indexOf(word.slug);
					if (newSlugIndex !== -1) newSlugs.splice(newSlugIndex, 1);
				} else {
					existing.quantity = next;
					existing.slug ??= word.slug;
					if (actual > 0 && word.market_median != null) {
						existing.marketMedian = word.market_median;
						existing.marketMedianUsesOfferFallback = word.market_median_from_current_offers;
					}
					existing.ducats ??= word.ducats;
				}
			} else {
				if (!newSlugs.includes(word.slug)) newSlugs.push(word.slug);
				items.push({
					name: word.text,
					slug: word.slug,
					isCustom: word.is_custom,
					quantity: next,
					marketMedian: word.market_median,
					marketMedianUsesOfferFallback: word.market_median_from_current_offers,
					ducats: word.ducats,
				});
			}
			applied.set(word.slug, (applied.get(word.slug) ?? 0) + actual);
		}
		if (applied.size === 0) return applied;
		await store.set('items', items);
		await store.set('newSlugs', newSlugs);
		await store.save();
		return applied;
	});
	latestSave = save.then(() => undefined);
	return save;
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
