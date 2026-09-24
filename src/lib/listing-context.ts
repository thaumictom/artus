import { inventoryMarketSlug, inventoryNameKey, type InventoryItem } from '$lib/inventory';
import type { MasteryItem } from '$lib/mastery.svelte';

export function ownedMarketCount(items: InventoryItem[], slug: string | undefined, name: string) {
	const nameKey = inventoryNameKey(name);
	return items.reduce((count, item) => {
		if (item.isCustom || item.quantity <= 0) return count;
		const itemSlug = inventoryMarketSlug(item);
		return count + (slug && itemSlug === slug || !itemSlug && inventoryNameKey(item.name) === nameKey ? item.quantity : 0);
	}, 0);
}

export function masteredMarketItems(items: MasteryItem[], checkedKeys: string[]) {
	const checked = new Set(checkedKeys);
	const slugs = new Set<string>();
	const names = new Set<string>();
	for (const item of items) {
		if (checked.has(item.key)) {
			if (item.marketSlug) slugs.add(item.marketSlug);
			names.add(inventoryNameKey(item.name));
		}
		for (const component of item.components) {
			if (!checked.has(item.key) && !checked.has(component.key)) continue;
			if (component.marketSlug) slugs.add(component.marketSlug);
			names.add(inventoryNameKey(`${item.name} ${component.name}`));
		}
	}
	return { slugs, names };
}

export function isMarketItemMastered(index: ReturnType<typeof masteredMarketItems>, slug: string | undefined, name: string) {
	return slug ? index.slugs.has(slug) : index.names.has(inventoryNameKey(name));
}
