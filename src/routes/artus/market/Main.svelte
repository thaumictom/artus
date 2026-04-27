<script lang="ts">
	import Combobox from '$lib/components/Combobox.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	type MarketDictionaryItem = {
		label: string;
		value: string;
	};

	type MarketDictionaryEntry = {
		name: string;
		slug: string;
	};

	type MarketOrder = {
		id: string;
		type: 'buy' | 'sell';
		platinum: number;
		quantity: number;
		perTrade: number;
		visible: boolean;
		createdAt: string;
		updatedAt: string;
		itemId: string;
		user: {
			id: string;
			ingameName: string;
			slug: string;
			reputation: number;
			platform: string;
			crossplay: boolean;
			status: string;
			lastSeen: string;
		};
	};

	type MarketResponse = {
		apiVersion: string;
		data: {
			sell: MarketOrder[];
			buy: MarketOrder[];
		};
	};

	let dictionaryItems = $state<MarketDictionaryItem[]>([]);
	let isLoadingDictionary = $state(true);
	let isSearching = $state(false);
	let statusMessage = $state<string | null>(null);
	let selectedSlug = $state<string | null>(null);
	let marketData = $state<MarketResponse | null>(null);

	async function loadDictionaryItems() {
		isLoadingDictionary = true;
		statusMessage = null;

		try {
			const dictionaryEntries = await invoke<MarketDictionaryEntry[]>(
				'get_market_dictionary_items',
			);
			dictionaryItems = dictionaryEntries.map((entry) => ({
				label: entry.name,
				value: entry.slug,
			}));
			if (dictionaryItems.length === 0) {
				statusMessage = 'No dictionary items are loaded yet.';
			}
		} catch (error) {
			statusMessage = String(error);
		} finally {
			isLoadingDictionary = false;
		}
	}

	async function fetchBySlug(slug: string) {
		if (isSearching || isLoadingDictionary) {
			return;
		}

		const dictionaryItem = dictionaryItems.find((item) => item.value === slug);
		if (!dictionaryItem) {
			selectedSlug = null;
			marketData = null;
			statusMessage = 'No matching dictionary item found.';
			return;
		}

		isSearching = true;
		selectedSlug = dictionaryItem.value;
		statusMessage = null;

		try {
			const responsePayload = await invoke<MarketResponse>('fetch_market_item_by_slug', {
				slug: dictionaryItem.value,
			});
			marketData = responsePayload;
			statusMessage = `Fetched market payload for ${dictionaryItem.label}.`;
		} catch (error) {
			marketData = null;
			statusMessage = String(error);
		} finally {
			isSearching = false;
		}
	}

	function handleValueChange(nextValue: string) {
		if (!nextValue) {
			selectedSlug = null;
			marketData = null;
			statusMessage = null;
			return;
		}

		void fetchBySlug(nextValue);
	}

	onMount(() => {
		void loadDictionaryItems();
	});
</script>

<div class="flex mx-auto p-8 w-full max-w-2xl">
	<div class="flex flex-col gap-1 w-full">
		<h1>View prices of any item on warframe.market</h1>
		<Combobox
			onValueChange={handleValueChange}
			type="single"
			items={dictionaryItems}
			disabled={isLoadingDictionary || isSearching}
			inputProps={{ placeholder: 'Search for an item...' }}
		></Combobox>
	</div>
</div>

{#if statusMessage}
	<p class="mt-2 text-muted-foreground text-sm">{statusMessage}</p>
{/if}

{#if marketData}
	<div class="space-y-8 mt-6 px-8">
		<!-- Sell Orders -->
		<div>
			<h2 class="mb-4 font-semibold text-lg">Sell Orders (Top 5 Cheapest)</h2>
			{#if marketData.data.sell.length > 0}
				<div class="border rounded-lg overflow-x-auto">
					<table class="w-full text-sm">
						<thead class="bg-muted border-b">
							<tr>
								<th class="px-4 py-3 font-medium text-left">Price</th>
								<th class="px-4 py-3 font-medium text-left">Qty</th>
								<th class="px-4 py-3 font-medium text-left">Seller</th>
								<th class="px-4 py-3 font-medium text-left">Rep</th>
								<th class="px-4 py-3 font-medium text-left">Platform</th>
								<th class="px-4 py-3 font-medium text-left">Updated</th>
								<th class="px-4 py-3 font-medium text-left">Status</th>
							</tr>
						</thead>
						<tbody>
							{#each marketData.data.sell.slice(0, 5) as order (order.id)}
								<tr class="hover:bg-muted/50 border-b">
									<td class="px-4 py-3 font-semibold text-green-600">{order.platinum} Pt</td>
									<td class="px-4 py-3">{order.quantity}</td>
									<td class="px-4 py-3 text-blue-500">{order.user.ingameName}</td>
									<td class="px-4 py-3">{order.user.reputation}</td>
									<td class="px-4 py-3 capitalize">{order.user.platform}</td>
									<td class="px-4 py-3 text-muted-foreground text-xs">
										{new Date(order.updatedAt).toLocaleDateString()}
										{new Date(order.updatedAt).toLocaleTimeString()}
									</td>
									<td class="px-4 py-3">
										<span
											class="px-2 py-1 rounded font-medium text-xs"
											class:bg-green-100={order.user.status === 'online'}
											class:text-green-800={order.user.status === 'online'}
											class:bg-gray-100={order.user.status !== 'online'}
											class:text-gray-800={order.user.status !== 'online'}
										>
											{order.user.status}
										</span>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{:else}
				<p class="text-muted-foreground">No sell orders available.</p>
			{/if}
		</div>

		<!-- Buy Orders -->
		<div>
			<h2 class="mb-4 font-semibold text-lg">Buy Orders (Top 5 Highest)</h2>
			{#if marketData.data.buy.length > 0}
				<div class="border rounded-lg overflow-x-auto">
					<table class="w-full text-sm">
						<thead class="bg-muted border-b">
							<tr>
								<th class="px-4 py-3 font-medium text-left">Price</th>
								<th class="px-4 py-3 font-medium text-left">Qty</th>
								<th class="px-4 py-3 font-medium text-left">Buyer</th>
								<th class="px-4 py-3 font-medium text-left">Rep</th>
								<th class="px-4 py-3 font-medium text-left">Platform</th>
								<th class="px-4 py-3 font-medium text-left">Updated</th>
								<th class="px-4 py-3 font-medium text-left">Status</th>
							</tr>
						</thead>
						<tbody>
							{#each marketData.data.buy.slice(0, 5) as order (order.id)}
								<tr class="hover:bg-muted/50 border-b">
									<td class="px-4 py-3 font-semibold text-orange-600">{order.platinum} Pt</td>
									<td class="px-4 py-3">{order.quantity}</td>
									<td class="px-4 py-3 text-blue-500">{order.user.ingameName}</td>
									<td class="px-4 py-3">{order.user.reputation}</td>
									<td class="px-4 py-3 capitalize">{order.user.platform}</td>
									<td class="px-4 py-3 text-muted-foreground text-xs">
										{new Date(order.updatedAt).toLocaleDateString()}
										{new Date(order.updatedAt).toLocaleTimeString()}
									</td>
									<td class="px-4 py-3">
										<span
											class="px-2 py-1 rounded font-medium text-xs"
											class:bg-green-100={order.user.status === 'online'}
											class:text-green-800={order.user.status === 'online'}
											class:bg-gray-100={order.user.status !== 'online'}
											class:text-gray-800={order.user.status !== 'online'}
										>
											{order.user.status}
										</span>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{:else}
				<p class="text-muted-foreground">No buy orders available.</p>
			{/if}
		</div>
	</div>
{:else}
	<div class="mx-8 mt-4 p-3 border rounded text-muted-foreground text-sm">
		Select an item to fetch its prices from warframe.market.
	</div>
{/if}
