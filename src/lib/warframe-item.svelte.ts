import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { z } from 'zod';
import { untrack } from 'svelte';
import { inventory, initializeInventory } from '$lib/inventory.svelte';
import { catalogMarketSlug, MarketCatalogSchema, warframeItemName, type CatalogItem } from '$lib/market-catalog';
import { inventoryNameKey, inventoryMarketSlug } from '$lib/inventory';
import { marketAccount } from '$lib/market-account.svelte';
import { mastery } from '$lib/mastery.svelte';
import { masteredMarketItems, isMarketItemMastered } from '$lib/listing-context';
import { fetchMarketListings } from '$lib/market-listings';
import { appNavigation } from '$lib/app-navigation.svelte';
import type { Listing, ListingChange } from '../routes/artus/listings/types';

const identitySchema = z.object({
	id: z.string(), slug: z.string(), name: z.string(), gameRef: z.string().optional(),
	tags: z.array(z.string()).default([]), ducats: z.number().optional(),
});
type Identity = z.infer<typeof identitySchema>;
const identityCatalogSchema = z.object({ items: z.array(identitySchema) });

export function itemGameRef(reference: string): string {
	// Store entries and their underlying items have different game references.
	return reference.replace('/StoreItems/', '/');
}

class WarframeItems {
	// Catalogs are immutable snapshots; avoid creating deep proxies for every item.
	catalog = $state.raw<Record<string, CatalogItem>>({});
	bySlug = $state.raw<Record<string, Identity>>({});
	byGameRef = $state.raw<Record<string, Identity>>({});
	byId = $state.raw<Record<string, Identity>>({});
	addOptions = $state.raw<{ label: string; value: string; ducats?: number }[]>([]);
	listings = $state<Listing[]>([]);
	listingsLoaded = $state(false);
	listingsError = $state('');
	catalogReady = $state(false);
	catalogError = $state('');
	identitiesReady = $state(false);
	identitiesError = $state('');
}
export const warframeItems = new WarframeItems();

const ownedIndex = $derived.by(() => {
	const slugs = new Map<string, number>();
	const names = new Map<string, number>();
	for (const item of inventory.items) {
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
			warframeItems.byId = Object.fromEntries(identities.value.items.map((item) => [item.id, item]));
			warframeItems.addOptions = identities.value.items.map((item) => ({
				label: item.name, value: item.slug, ducats: item.ducats,
			}));
			warframeItems.identitiesReady = true;
			warframeItems.identitiesError = '';
			warframeItems.byGameRef = Object.fromEntries(identities.value.items.filter((item) => item.gameRef)
				.map((item) => [itemGameRef(item.gameRef!), item]));
		} else {
			warframeItems.identitiesError = 'Could not load market items.';
			console.error('Could not load shared market identities:', identities.reason);
		}
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

export async function refreshWarframeItemListings() {
	const account = marketAccount.session?.ingameName;
	const request = ++listingRequest;
	warframeItems.listingsError = '';
	if (!account) {
		warframeItems.listings = [];
		warframeItems.listingsLoaded = false;
	}
	if (!account) return;
	try {
		const listings = await fetchMarketListings();
		if (active && request === listingRequest && account === marketAccount.session?.ingameName) {
			warframeItems.listings = listings;
			warframeItems.listingsLoaded = true;
		}
	} catch (error) {
		if (active && request === listingRequest) warframeItems.listingsError = String(error);
		console.error('Could not load shared item listings:', error);
	}
}

export function recordWarframeItemListings(listings: Listing[]) {
	// A completed Listings refresh supersedes an older shared request.
	listingRequest++;
	warframeItems.listings = listings;
	warframeItems.listingsLoaded = true;
	warframeItems.listingsError = '';
}

export function applyWarframeListingChange(change: ListingChange) {
	switch (change.kind) {
		case 'created':
			if (change.listing) warframeItems.listings = [...warframeItems.listings, change.listing];
			else void refreshWarframeItemListings();
			break;
		case 'updated':
			warframeItems.listings = warframeItems.listings.map((listing) => listing.id === change.id
				? { ...listing, platinum: change.platinum, quantity: change.quantity } : listing);
			break;
		case 'visibility':
			warframeItems.listings = warframeItems.listings.map((listing) => listing.id === change.id
				? { ...listing, visible: change.visible } : listing);
			break;
		case 'deleted':
			warframeItems.listings = warframeItems.listings.filter((listing) => listing.id !== change.id);
			break;
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
	const stopInventory = initializeInventory();
	let listingsDirty = false;
	let listingTimer: ReturnType<typeof setTimeout> | undefined;
	const keep = (stop: UnlistenFn) => {
		if (!active || generation !== current) stop();
		else stops.push(stop);
	};
	void listen('api_catalogs_fetched', () => { void refreshWarframeItemCatalog(); }).then(keep)
		.catch((error) => console.error('Could not observe shared item catalog:', error));
	void listen('market_listings_changed', () => {
		// Listing rows already receive their live order; refresh the shared snapshot on leaving that tab.
		if (appNavigation.current.section === 'listings') {
			listingsDirty = true;
			return;
		}
		clearTimeout(listingTimer);
		listingTimer = setTimeout(() => { void refreshWarframeItemListings(); }, 500);
	}).then(keep).catch((error) => console.error('Could not observe shared item listings:', error));
	void refreshWarframeItemCatalog();
	const stopEffect = $effect.root(() => {
		$effect(() => {
			marketAccount.session?.ingameName;
			untrack(() => {
				warframeItems.listings = [];
				warframeItems.listingsLoaded = false;
				void refreshWarframeItemListings();
			});
		});
		$effect(() => {
			if (appNavigation.current.section !== 'listings' && listingsDirty) {
				listingsDirty = false;
				untrack(() => void refreshWarframeItemListings());
			}
		});
	});
	return () => {
		active = false;
		catalogRequest++;
		listingRequest++;
		stopEffect();
		stopInventory();
		clearTimeout(listingTimer);
		stops.forEach((stop) => stop());
	};
}
