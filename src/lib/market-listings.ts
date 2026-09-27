import { invoke } from '@tauri-apps/api/core';
import type { Listing } from '../routes/artus/listings/types';

/** Fetch the signed-in user's listings from the market account. */
export async function fetchMarketListings(): Promise<Listing[]> {
	const response = await invoke<{ data: Listing[] }>('market_my_orders');
	if (!Array.isArray(response.data)) {
		throw new Error('Could not read your market listings');
	}
	return response.data;
}
