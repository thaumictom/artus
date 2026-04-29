<script lang="ts">
	import Combobox from '$lib/components/Combobox.svelte';
	import { Slider } from 'bits-ui';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import {
		BarChart,
		Spline,
		Tooltip,
		defaultChartPadding,
		Axis,
		Layer,
		Highlight,
		Chart,
		Area,
	} from 'layerchart';
	import type { MarketStatEntry } from '$lib/types';
	import Select from '$lib/components/Select.svelte';

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
	let marketStats = $state<MarketStatEntry[]>([]);
	let rankRange = $state<[number, number]>([0, 0]);

	// Track the currently selected group (defaults to 0)
	let selectedModRank = $state<number>(0);

	// Extract unique mod_ranks to populate the <select> dropdown
	const availableGroups = $derived(
		Array.from(new Set(marketStats.map((s) => s.mod_rank ?? 0))).sort(),
	);

	const last28DaysStats = $derived.by(() => {
		if (marketStats.length === 0) return [];

		// 1. Filter out the duplicates by targeting only the selected group
		const filtered = marketStats.filter((s) => (s.mod_rank ?? 0) === selectedModRank);

		// 2. Sort the filtered array
		const sorted = [...filtered].sort(
			(a, b) => new Date(a.datetime).getTime() - new Date(b.datetime).getTime(),
		);

		// 3. Grab the last 28 days
		return sorted.slice(-28);
	});

	const chartData = $derived(
		last28DaysStats.map((s) => ({
			date: new Date(s.datetime),
			median: s.median,
			average: s.moving_avg,
			volume: s.volume,
		})),
	);

	$inspect(chartData, availableGroups);

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
			marketStats = [];
			statusMessage = 'No matching dictionary item found.';
			return;
		}

		isSearching = true;
		selectedSlug = dictionaryItem.value;
		statusMessage = null;

		try {
			const [responsePayload, statsPayload] = await Promise.all([
				invoke<MarketResponse>('fetch_market_item_by_slug', {
					slug: dictionaryItem.value,
				}),
				invoke<MarketStatEntry[]>('fetch_market_item_stats_by_slug', {
					slug: dictionaryItem.value,
				}),
			]);

			marketData = responsePayload;
			marketStats = statsPayload;

			const maxRank = getMaxRank(responsePayload.data) ?? 0;
			rankRange = [0, maxRank];
			statusMessage = `Fetched market payload for ${dictionaryItem.label}.`;
		} catch (error) {
			marketData = null;
			marketStats = [];
			statusMessage = String(error);
		} finally {
			isSearching = false;
		}
	}

	function handleValueChange(nextValue: string) {
		if (!nextValue) {
			selectedSlug = null;
			marketData = null;
			marketStats = [];
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

<div class="flex flex-col gap-8 mx-auto p-8 w-full max-w-2xl">
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

	{#if statusMessage}
		<p class="mt-2 text-muted-foreground text-sm text-center">{statusMessage}</p>
	{/if}

	{#if marketData}
		{@const maxRank = getMaxRank(marketData.data)}
		{@const hasRankData = maxRank !== undefined}
		{@const hasRankSlider = maxRank !== undefined && maxRank > 0}
		{@const rankMin = hasRankSlider ? Math.min(rankRange[0], rankRange[1]) : 0}
		{@const rankMax = hasRankSlider ? Math.max(rankRange[0], rankRange[1]) : (maxRank ?? 0)}
		{@const filteredOrders = filterOrdersForDisplay(
			marketData.data,
			hasRankSlider,
			rankMin,
			rankMax,
		)}
		{@const sellOrders = getTopSellOrders(filteredOrders.filter((order) => order.type === 'sell'))}
		{@const buyOrders = getTopBuyOrders(filteredOrders.filter((order) => order.type === 'buy'))}
		<div class="border">
			<div>Name: {marketData?.data[0]?.itemId}</div>
			<div>Image:</div>
			<div>Ducats:</div>
		</div>
		<div class="space-y-8 mt-6">
			{#if chartData.length > 0}
				<select bind:value={selectedModRank}>
					{#each availableGroups as group}
						<option value={group}>Mod Rank: {group}</option>
					{/each}
				</select>
				<div class="grid p-4 border">
					<div class="col-start-1 row-start-1">
						<BarChart
							data={chartData}
							x="date"
							y="volume"
							yNice
							axis={false}
							grid={false}
							props={{
								bars: { radius: 1, class: 'stroke-none fill-surface' },
							}}
							padding={defaultChartPadding({ left: 25 })}
							height={300}
						></BarChart>
					</div>
					<div class="col-start-1 row-start-1">
						<BarChart
							data={chartData}
							x="date"
							y={['median', 'average']}
							yNice
							yDomain={null}
							height={300}
							props={{
								xAxis: { ticks: 10, rule: true },
								tooltip: { context: { mode: 'band' } },
							}}
							padding={defaultChartPadding({ left: 25 })}
						>
							{#snippet marks()}
								<Spline y="median" class="stroke-blue-500" />
								<Spline y="average" class="stroke-red-500" />
							{/snippet}

							{#snippet tooltip({ context })}
								<Tooltip.Root {context}>
									{#snippet children({ data })}
										<Tooltip.Header value={data.date} format="daytime" />
										<Tooltip.List>
											<Tooltip.Item label="median" value={data.median} format="integer" />
											<Tooltip.Item label="moving average" value={data.average} format="decimal" />
											<Tooltip.Item label="volume" value={data.volume} format="integer" />
										</Tooltip.List>
									{/snippet}
								</Tooltip.Root>
							{/snippet}
						</BarChart>
					</div>
				</div>
			{/if}

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
							type="multiple"
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
	{/if}
</div>
