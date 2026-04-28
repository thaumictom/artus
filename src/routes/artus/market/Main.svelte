<script lang="ts">
	import Combobox from '$lib/components/Combobox.svelte';
	import { Slider } from 'bits-ui';
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
		rank?: number;
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
		data: MarketOrder[];
	};

	let dictionaryItems = $state<MarketDictionaryItem[]>([]);
	let isLoadingDictionary = $state(true);
	let isSearching = $state(false);
	let statusMessage = $state<string | null>(null);
	let selectedSlug = $state<string | null>(null);
	let marketData = $state<MarketResponse | null>(null);
	let rankRange = $state<[number, number]>([0, 0]);

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
			const maxRank = getMaxRank(responsePayload.data) ?? 0;
			rankRange = [0, maxRank];
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

	function getTopSellOrders(orders: MarketOrder[]): MarketOrder[] {
		return [...orders].sort((left, right) => left.platinum - right.platinum).slice(0, 10);
	}

	function getTopBuyOrders(orders: MarketOrder[]): MarketOrder[] {
		return [...orders].sort((left, right) => right.platinum - left.platinum).slice(0, 10);
	}

	function getOrderRank(order: MarketOrder): number | undefined {
		return typeof order.rank === 'number' ? order.rank : undefined;
	}

	function getMaxRank(orders: MarketOrder[]): number | undefined {
		const ranks = orders
			.map((order) => getOrderRank(order))
			.filter((value): value is number => typeof value === 'number');
		if (ranks.length === 0) {
			return undefined;
		}
		return Math.max(...ranks);
	}

	function isIngameOrder(order: MarketOrder): boolean {
		return order.user.status === 'ingame';
	}

	function filterOrdersForDisplay(
		orders: MarketOrder[],
		filterByRank: boolean,
		minRank: number,
		maxRank: number,
	): MarketOrder[] {
		return orders.filter((order) => {
			if (!isIngameOrder(order)) {
				return false;
			}

			if (!filterByRank) {
				return true;
			}

			const rank = getOrderRank(order);
			if (rank === undefined) {
				return false;
			}
			return rank >= minRank && rank <= maxRank;
		});
	}

	function formatOrderTimestamp(value: string): string {
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) {
			return value;
		}
		return `${date.toLocaleDateString()} ${date.toLocaleTimeString()}`;
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
	{@const maxRank = getMaxRank(marketData.data)}
	{@const hasRankData = maxRank !== undefined}
	{@const hasRankSlider = maxRank !== undefined && maxRank > 0}
	{@const rankMin = hasRankSlider ? Math.min(rankRange[0], rankRange[1]) : 0}
	{@const rankMax = hasRankSlider ? Math.max(rankRange[0], rankRange[1]) : (maxRank ?? 0)}
	{@const filteredOrders = filterOrdersForDisplay(marketData.data, hasRankSlider, rankMin, rankMax)}
	{@const sellOrders = getTopSellOrders(filteredOrders.filter((order) => order.type === 'sell'))}
	{@const buyOrders = getTopBuyOrders(filteredOrders.filter((order) => order.type === 'buy'))}
	<div class="space-y-8 mt-6 px-8">
		{#if hasRankSlider}
			<div class="flex flex-col gap-3 max-w-md">
				<div class="flex justify-between items-center gap-3">
					<div>
						<p class="font-medium">Rank range</p>
						<p class="text-muted-foreground text-xs">
							Filter between rank {rankMin} and {rankMax} (max {maxRank}).
						</p>
					</div>
					<div class="tabular-nums text-sm">
						{rankMin} - {rankMax}
					</div>
				</div>
				<div class="group flex items-center h-6">
					<Slider.Root
						bind:value={rankRange}
						min={0}
						max={maxRank}
						step={1}
						thumbPositioning="equal"
						trackPadding={1.5}
						class="relative flex items-center bg-surface border w-full h-1.5 has-data-active:h-2.5 group-hover:h-2.5 transition-[height] touch-none select-none"
					>
						<Slider.Range class="absolute bg-foreground h-full" />
						<Slider.Thumb index={0} class="group">
							<div
								class="bg-foreground border size-4.5 group-data-active:size-5.5 transition-all cursor-e-resize"
							></div>
						</Slider.Thumb>
						<Slider.Thumb index={1} class="group">
							<div
								class="bg-foreground border size-4.5 group-data-active:size-5.5 transition-all cursor-e-resize"
							></div>
						</Slider.Thumb>
					</Slider.Root>
				</div>
			</div>
		{/if}
		<div>
			<h2 class="mb-4 font-semibold text-lg">Sell Orders (Top 10 Cheapest)</h2>
			{#if sellOrders.length > 0}
				<div class="border rounded-lg overflow-x-auto">
					<table class="w-full text-sm">
						<thead class="bg-muted border-b">
							<tr>
								<th class="px-4 py-3 font-medium text-left">Price</th>
								<th class="px-4 py-3 font-medium text-left">Qty</th>
								<th class="px-4 py-3 font-medium text-left">Per Trade</th>
								{#if hasRankData}
									<th class="px-4 py-3 font-medium text-left">Rank</th>
								{/if}
								<th class="px-4 py-3 font-medium text-left">Seller</th>
								<th class="px-4 py-3 font-medium text-left">Rep</th>
								<th class="px-4 py-3 font-medium text-left">Platform</th>
								<th class="px-4 py-3 font-medium text-left">Updated</th>
								<th class="px-4 py-3 font-medium text-left">Status</th>
							</tr>
						</thead>
						<tbody>
							{#each sellOrders as order (order.id)}
								{@const rankValue = getOrderRank(order) ?? 0}
								<tr class="hover:bg-muted/50 border-b">
									<td class="px-4 py-3 font-semibold text-green-600">
										{order.platinum} Pt
									</td>
									<td class="px-4 py-3">{order.quantity}</td>
									<td class="px-4 py-3">{order.perTrade}</td>
									{#if hasRankData}
										<td class="px-4 py-3">{rankValue} of {maxRank ?? 0}</td>
									{/if}
									<td class="px-4 py-3 text-blue-500">{order.user.ingameName}</td>
									<td class="px-4 py-3">{order.user.reputation}</td>
									<td class="px-4 py-3 capitalize">{order.user.platform}</td>
									<td class="px-4 py-3 text-muted-foreground text-xs">
										{formatOrderTimestamp(order.updatedAt)}
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

		<div>
			<h2 class="mb-4 font-semibold text-lg">Buy Orders (Top 10 Highest)</h2>
			{#if buyOrders.length > 0}
				<div class="border rounded-lg overflow-x-auto">
					<table class="w-full text-sm">
						<thead class="bg-muted border-b">
							<tr>
								<th class="px-4 py-3 font-medium text-left">Price</th>
								<th class="px-4 py-3 font-medium text-left">Qty</th>
								<th class="px-4 py-3 font-medium text-left">Per Trade</th>
								{#if hasRankData}
									<th class="px-4 py-3 font-medium text-left">Rank</th>
								{/if}
								<th class="px-4 py-3 font-medium text-left">Buyer</th>
								<th class="px-4 py-3 font-medium text-left">Rep</th>
								<th class="px-4 py-3 font-medium text-left">Platform</th>
								<th class="px-4 py-3 font-medium text-left">Updated</th>
								<th class="px-4 py-3 font-medium text-left">Status</th>
							</tr>
						</thead>
						<tbody>
							{#each buyOrders as order (order.id)}
								{@const rankValue = getOrderRank(order) ?? 0}
								<tr class="hover:bg-muted/50 border-b">
									<td class="px-4 py-3 font-semibold text-orange-600">
										{order.platinum} Pt
									</td>
									<td class="px-4 py-3">{order.quantity}</td>
									<td class="px-4 py-3">{order.perTrade}</td>
									{#if hasRankData}
										<td class="px-4 py-3">{rankValue} of {maxRank ?? 0}</td>
									{/if}
									<td class="px-4 py-3 text-blue-500">{order.user.ingameName}</td>
									<td class="px-4 py-3">{order.user.reputation}</td>
									<td class="px-4 py-3 capitalize">{order.user.platform}</td>
									<td class="px-4 py-3 text-muted-foreground text-xs">
										{formatOrderTimestamp(order.updatedAt)}
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
