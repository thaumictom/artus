import type { InventoryItem } from '$lib/inventory';
import { ownedMarketCount } from '$lib/listing-context';
import type { Listing, ListingItem } from './types';

export function listingItem(order: Listing, details: Record<string, ListingItem>): ListingItem | undefined {
	return details[order.itemId];
}

export function listingName(order: Listing, details: Record<string, ListingItem>): string {
	return listingItem(order, details)?.name ?? order.itemId;
}

export function listingSlug(order: Listing, details: Record<string, ListingItem>): string {
	return listingItem(order, details)?.slug ?? order.itemId;
}

export function listingOwned(order: Listing, details: Record<string, ListingItem>, inventory: InventoryItem[]): number {
	return ownedMarketCount(inventory, listingItem(order, details)?.slug, listingName(order, details));
}
