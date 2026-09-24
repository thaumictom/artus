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
