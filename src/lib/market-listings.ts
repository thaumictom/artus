import { invoke } from '@tauri-apps/api/core';
import type { Listing } from '../routes/artus/listings/types';
import { marketAccount } from '$lib/market-account.svelte';

let pending: { account: string | undefined; promise: Promise<Listing[]> } | null = null;

/** Fetch the signed-in user's listings from the market account. */
export function fetchMarketListings(): Promise<Listing[]> {
	const account = marketAccount.session?.ingameName;
	// The shared item context and a mounted tab can request the same snapshot together.
	if (pending && pending.account === account) return pending.promise;
	const promise = invoke<{ data: Listing[] }>('market_my_orders').then((response) => {
		if (!Array.isArray(response.data)) throw new Error('Could not read your market listings');
		return response.data;
	}).finally(() => {
		if (pending?.promise === promise) pending = null;
	});
	pending = { account, promise };
	return promise;
}
