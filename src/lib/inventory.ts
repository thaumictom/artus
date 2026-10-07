import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type InventoryItem = {
	name: string;
	slug?: string;
	isCustom?: boolean;
	category?: string;
	quantity: number;
	ducats?: number;
};

export type InventoryOcrWord = {
	slug?: string;
	is_custom?: boolean;
	text: string;
	ducats?: number;
};

export type InventorySnapshot = {
	revision: number;
	items: InventoryItem[];
	newSlugs: string[];
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

// These are IPC view models. Rust owns quantities, persistence and mutation ordering.
export function getInventorySnapshot() {
	return invoke<InventorySnapshot>('inventory_snapshot');
}

export function watchInventory(listener: (snapshot: InventorySnapshot) => void) {
	return listen<InventorySnapshot>('inventory_changed', ({ payload }) => listener(payload));
}

type QuantityResult = { snapshot: InventorySnapshot; applied: Record<string, number> };
type OcrQuantityChange = { word: InventoryOcrWord; delta: number } | { word: InventoryOcrWord; quantity: number };

async function applyOcrQuantities(changes: OcrQuantityChange[]) {
	const result = await invoke<QuantityResult>('inventory_change_ocr_quantities', { changes });
	return new Map(Object.entries(result.applied));
}

export function changeOcrItemQuantities(changes: { word: InventoryOcrWord; delta: number }[]) {
	return applyOcrQuantities(changes);
}

export function setOcrItemQuantities(changes: { word: InventoryOcrWord; quantity: number }[]) {
	return applyOcrQuantities(changes);
}

export async function addInventoryItem(word: InventoryOcrWord, quantity: number) {
	await invoke<InventorySnapshot>('inventory_add_item', { word, quantity });
}

export async function changeInventoryRowQuantity(item: InventoryItem, delta: number) {
	await invoke<InventorySnapshot>('inventory_change_row_quantity', {
		slug: item.slug ?? null, name: item.name, isCustom: item.isCustom ?? false, delta,
	});
}

export async function dismissNewInventoryItems() {
	await invoke<InventorySnapshot>('inventory_dismiss_new_items');
}

export function removeOneMarketInventoryItem(slug: string | undefined, name: string) {
	return changeMarketInventoryQuantity(slug, name, -1);
}

export async function changeMarketInventoryQuantity(slug: string | undefined, name: string, delta: number) {
	if (!Number.isSafeInteger(delta) || delta === 0) return;
	await invoke<InventorySnapshot>('inventory_change_market_quantity', { slug: slug ?? null, name, delta });
}

export async function resetInventory() {
	await invoke<InventorySnapshot>('inventory_reset');
}
