import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { LazyStore } from '@tauri-apps/plugin-store';
import { z } from 'zod';
import { catalogMarketSlug, MarketCatalogSchema, warframeItemName, type CatalogItem } from '$lib/market-catalog';
import { inventoryNameKey, inventoryMarketSlug, waitForInventorySave, type InventoryItem } from '$lib/inventory';
import { marketAccount } from '$lib/market-account.svelte';
import { mastery } from '$lib/mastery.svelte';
import { masteredMarketItems, isMarketItemMastered } from '$lib/listing-context';
import { fetchMarketListings } from '$lib/market-listings';
import { appNavigation } from '$lib/app-navigation.svelte';
import type { Listing } from '../routes/artus/listings/types';

const identitySchema = z.object({
	id: z.string(), slug: z.string(), name: z.string(), gameRef: z.string().optional(),
});
type Identity = z.infer<typeof identitySchema>;
const identityCatalogSchema = z.object({ items: z.array(identitySchema) });
const inventoryStore = new LazyStore('inventory.json');

export function itemGameRef(reference: string): string {
	// Store entries and their underlying items have different game references.
	return reference.replace('/StoreItems/', '/');
}

export const warframeItems = $state({
	catalog: {} as Record<string, CatalogItem>,
	bySlug: {} as Record<string, Identity>,
	byGameRef: {} as Record<string, Identity>,
	inventory: [] as InventoryItem[],
	listings: [] as Listing[],
	catalogReady: false,
	catalogError: '',
});

const ownedIndex = $derived.by(() => {
	const slugs = new Map<string, number>();
	const names = new Map<string, number>();
	for (const item of warframeItems.inventory) {
		if (item.isCustom || item.quantity <= 0) continue;
		const slug = inventoryMarketSlug(item);
		const index = slug ? slugs : names;
		const key = slug ?? inventoryNameKey(item.name);
		index.set(key, (index.get(key) ?? 0) + item.quantity);
	}
	return { slugs, names };
});
const masteredIndex = $derived(masteredMarketItems(mastery.items, mastery.checked));
const listingsIndex = $derived.by(() => {
	const index = new Map<string, Listing>();
	for (const listing of warframeItems.listings) {
		if (listing.type !== 'sell') continue;
		const previous = index.get(listing.itemId);
		if (!previous || (!previous.visible && listing.visible)) index.set(listing.itemId, listing);
	}
	return index;
});

export function resolveWarframeItem(reference: string, fallbackName = '') {
	const gameRef = reference.startsWith('/') ? itemGameRef(reference) : undefined;
	const identity = gameRef ? warframeItems.byGameRef[gameRef] : warframeItems.bySlug[reference];
	const key = gameRef ?? (identity?.gameRef ? itemGameRef(identity.gameRef) : undefined);
	const metadata = key ? warframeItems.catalog[key] : undefined;
	const slug = identity?.slug || catalogMarketSlug(metadata);
	const marketIdentity = slug ? warframeItems.bySlug[slug] ?? identity : identity;
	const name = warframeItemName(metadata, marketIdentity?.name, fallbackName) || reference;
	return {
		name, slug, gameRef: key, metadata,
		ownedCount: (slug ? ownedIndex.slugs.get(slug) ?? 0 : 0) + (ownedIndex.names.get(inventoryNameKey(name)) ?? 0),
		mastered: Boolean(key && mastery.checked.includes(key)) || isMarketItemMastered(masteredIndex, slug, name),
		listing: marketIdentity ? listingsIndex.get(marketIdentity.id) : undefined,
	};
}

export function itemSubtext(metadata?: CatalogItem): string {
	if (!metadata) return '';
	const summary = [
		metadata.type || metadata.category,
		metadata.masteryReq != null ? `MR ${metadata.masteryReq}` : undefined,
		metadata.compatName,
	].filter(Boolean);
	return [...new Set(summary)].join(' · ');
}

let active = false;
let consumers = 0;
let stopContext: (() => void) | undefined;
let generation = 0;
let catalogRequest = 0;
let listingRequest = 0;

export async function refreshWarframeItemCatalog() {
	const request = ++catalogRequest;
	try {
		const [catalog, identities] = await Promise.allSettled([
			invoke('get_cached_market_items').then((value) => MarketCatalogSchema.parse(value)),
			invoke('get_market_dictionary').then((value) => identityCatalogSchema.parse(value)),
		]);
		if (!active || request !== catalogRequest) return;
		if (identities.status === 'fulfilled') {
			warframeItems.bySlug = Object.fromEntries(identities.value.items.map((item) => [item.slug, item]));
			warframeItems.byGameRef = Object.fromEntries(identities.value.items.filter((item) => item.gameRef)
				.map((item) => [itemGameRef(item.gameRef!), item]));
		} else console.error('Could not load shared market identities:', identities.reason);
		// Cached game metadata remains useful even while the market identity feed is unavailable.
		if (catalog.status === 'rejected') throw catalog.reason;
		warframeItems.catalog = catalog.value;
		warframeItems.catalogReady = true;
		warframeItems.catalogError = '';
	} catch (error) {
		if (!active || request !== catalogRequest) return;
		warframeItems.catalogError = 'Item details could not be loaded.';
		console.error('Could not load shared item catalog:', error);
	}
}

async function refreshListings() {
	const account = marketAccount.session?.ingameName;
	const request = ++listingRequest;
	warframeItems.listings = [];
	if (!account) return;
	try {
		const listings = await fetchMarketListings();
		if (active && request === listingRequest && account === marketAccount.session?.ingameName)
			warframeItems.listings = listings;
	} catch (error) {
		console.error('Could not load shared item listings:', error);
	}
}

/** One set of reads and listeners per desktop window, never per item component. */
export function initializeWarframeItems() {
	consumers++;
	if (!active) stopContext = startWarframeItems();
	let released = false;
	return () => {
		if (released) return;
		released = true;
		if (--consumers === 0) {
			stopContext?.();
			stopContext = undefined;
		}
	};
}

function startWarframeItems() {
	active = true;
	const current = ++generation;
	const stops: UnlistenFn[] = [];
	let listingTimer: ReturnType<typeof setTimeout> | undefined;
	const keep = (stop: UnlistenFn) => {
		if (!active || generation !== current) stop();
		else stops.push(stop);
	};
	void inventoryStore.onChange<unknown>((key, value) => {
		if (key === 'items' && Array.isArray(value)) warframeItems.inventory = value as InventoryItem[];
	}).then(keep).catch((error) => console.error('Could not observe shared inventory:', error));
	void waitForInventorySave().catch(() => undefined).then(() => inventoryStore.get<InventoryItem[]>('items'))
		.then((items) => { if (active && current === generation) warframeItems.inventory = items ?? []; })
		.catch((error) => console.error('Could not read shared inventory:', error));
	void listen('api_catalogs_fetched', () => { void refreshWarframeItemCatalog(); }).then(keep)
		.catch((error) => console.error('Could not observe shared item catalog:', error));
	void listen('market_listings_changed', () => {
		// Listing rows already receive their live order; refresh the shared snapshot on leaving that tab.
		if (appNavigation.current.section === 'listings') return;
		clearTimeout(listingTimer);
		listingTimer = setTimeout(() => { void refreshListings(); }, 500);
	}).then(keep).catch((error) => console.error('Could not observe shared item listings:', error));
	void refreshWarframeItemCatalog();
	const stopEffect = $effect.root(() => {
		$effect(() => {
			marketAccount.session?.ingameName;
			appNavigation.current.section;
			void refreshListings();
		});
	});
	return () => {
		active = false;
		catalogRequest++;
		listingRequest++;
		stopEffect();
		clearTimeout(listingTimer);
		stops.forEach((stop) => stop());
	};
}
