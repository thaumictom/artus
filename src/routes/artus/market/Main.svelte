<script lang="ts">
	import Combobox from '$lib/components/Combobox.svelte';
	import Button from '$lib/components/Button.svelte';
	// import { RadioGroup } from 'bits-ui'; // Tag filtering is temporarily disabled.
	import { onMount } from 'svelte';
	import { z } from 'zod';
	import {
		ItemSchema,
		DictionarySchema,
		GetItemResponseSchema,
		MostTradedItemsSchema,
	} from '$lib/schemas';
	import Orders from './Orders.svelte';
	import Statistics from './Statistics.svelte';
	import InfoCard from './InfoCard.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { MarketCatalogSchema, type CatalogItem } from '$lib/market-catalog';

	let catalog = $state.raw<Record<string, CatalogItem> | null>(null);
	let catalogError = $state(false);
	let dictionaryItems: {
		label: string;
		value: string;
		gameRef?: string;
		setSlug?: string | null;
		isSet: boolean;
		tags: string[];
	}[] = $state([]);
	let mostTradedItems = $state<z.infer<typeof MostTradedItemsSchema>>([]);
	let visibleItemCount = $state(20);
	/* Tag filtering is temporarily disabled; retain it for re-enabling later.
	let selectedTag = $state('__all__');
	let tagsBySlug = $derived(new Map(dictionaryItems.map((item) => [item.value, item.tags])));
	let tagFilters = $derived.by(() => {
		const totals = new Map<string, { liquidity: number; count: number }>();
		// Aggregate the full ranking before filtering or limiting the visible items.
		for (const item of mostTradedItems) {
			for (const tag of new Set(tagsBySlug.get(item.slug) ?? [])) {
				if (!tag) continue;
				const total = totals.get(tag) ?? { liquidity: 0, count: 0 };
				totals.set(tag, { liquidity: total.liquidity + item.liquidity, count: total.count + 1 });
			}
		}
		return [...totals].map(([tag, total]) => ({ tag, ...total }))
			.filter(({ count }) => count >= 12)
			.sort((a, b) => b.liquidity - a.liquidity || a.tag.localeCompare(b.tag));
	});
	let filteredMostTradedItems = $derived(selectedTag !== '__all__'
		? mostTradedItems.filter((item) => tagsBySlug.get(item.slug)?.includes(selectedTag))
		: mostTradedItems);
	*/
	let filteredMostTradedItems = $derived(mostTradedItems);
	let visibleMostTradedItems = $derived(filteredMostTradedItems.slice(0, visibleItemCount));
	let isLoadingMostTraded = $state(true);
	let mostTradedError = $state(false);
	let disposed = false;
	let searchSequence = 0;
	const number = new Intl.NumberFormat();

	async function loadMostTraded() {
		isLoadingMostTraded = true;
		mostTradedError = false;
		try {
			const response = await invoke('get_most_traded_items');
			if (!disposed) mostTradedItems = MostTradedItemsSchema.parse(response);
		} catch (error) {
			if (!disposed) {
				console.error('Failed to load most traded items:', error);
				mostTradedError = true;
			}
		} finally {
			if (!disposed) isLoadingMostTraded = false;
		}
	}

	let isLoadingDictionary = $state(false);
	let isSearching = $state(false);
	let dictionaryError = $state<string | null>(null);

	let selectedSlug = $state('');
	let searchError = $state<string | null>(null);
	let relatedItems = $derived.by(() => {
		const current = dictionaryItems.find((item) => item.value === itemData?.slug);
		const setSlug = current?.isSet ? current.value : current?.setSlug;
		if (!setSlug) return [];
		const setName = dictionaryItems
			.find((item) => item.value === setSlug)
			?.label.replace(/ Set$/, '');
		const order = (label: string) => (label === 'Set' ? 0 : label === 'Blueprint' ? 1 : 2);
		return dictionaryItems
			.filter((item) => item.value === setSlug || item.setSlug === setSlug)
			.map((item) => ({
				value: item.value,
				itemCount: !item.isSet && item.gameRef ? catalog?.[item.gameRef]?.itemCount : undefined,
				label:
					item.value === setSlug
						? 'Set'
						: setName && item.label.startsWith(`${setName} `)
							? item.label.slice(setName.length + 1)
							: item.label,
			}))
			.map((item) => ({ ...item, label: item.label.replace(/ Blueprint$/, '') }))
			.sort((a, b) => order(a.label) - order(b.label) || a.label.localeCompare(b.label));
	});

	async function loadDictionary() {
		isLoadingDictionary = true;
		dictionaryError = null;
		try {
			const response = await invoke('get_market_dictionary');
			const data = DictionarySchema.parse(response);
			dictionaryItems = data.items.map((item) => ({
				label: item.name,
				value: item.slug,
				gameRef: item.gameRef,
				setSlug: item.set_slug,
				isSet: item.tags.includes('set'),
				tags: item.tags,
			}));
		} catch (err) {
			console.error('Failed to load dictionary:', err);
			dictionaryError = 'Could not load the item list. Please try again.';
		} finally {
			isLoadingDictionary = false;
		}
	}

	onMount(() => {
		void loadDictionary();
		void loadMostTraded();
		void invoke('get_cached_market_items')
			.then((response) => {
				if (!disposed) catalog = MarketCatalogSchema.parse(response);
			})
			.catch((error) => {
				if (!disposed) {
					console.error('Failed to read local item catalog:', error);
					catalogError = true;
				}
			});
		return () => {
			disposed = true;
			catalog = null;
		};
	});

	let itemData: z.infer<typeof ItemSchema> | null = $state(null);

	function closeItem() {
		// Ignore pending detail requests after returning to the landing page.
		searchSequence += 1;
		itemData = null;
		selectedSlug = '';
		searchError = null;
		isSearching = false;
	}

	const handleValueChange = (slug: string) => {
		if (!slug || isSearching || slug === itemData?.slug) return;
		isSearching = true;
		searchError = null;
		const sequence = ++searchSequence;

		invoke('get_market_item', { slug })
			.then((response: any) => {
				if (disposed || sequence !== searchSequence) return;
				const { data } = GetItemResponseSchema.parse(response);
				itemData = data;
				selectedSlug = data.slug;
			})
			.catch((err) => {
				if (disposed || sequence !== searchSequence) return;
				console.error('Search failed:', err);
				searchError = 'Could not load this item. Please try again.';
			})
			.finally(() => {
				if (!disposed && sequence === searchSequence) isSearching = false;
			});
	};
</script>

<div class="flex flex-col items-center gap-4 mx-auto p-8 w-full">
	<div class="flex flex-col gap-1 w-full max-w-3xl">
		<h1>View prices of any item on warframe.market</h1>
		<div class="flex gap-4 w-full">
			<div class="flex-1">
				<Combobox
					onValueChange={handleValueChange}
					type="single"
					items={dictionaryItems}
					bind:value={selectedSlug}
					disabled={isLoadingDictionary || isSearching}
					inputProps={{
						placeholder: isLoadingDictionary ? 'Loading items...' : 'Search for an item...',
					}}
				></Combobox>
			</div>
			{#if itemData}
				<div class="flex-shrink-0">
					<Button onclick={closeItem} class="h-full">Close</Button>
				</div>
			{/if}
		</div>
		{#if dictionaryError}
			<div role="alert" class="flex items-center gap-2 text-sm">
				<span>{dictionaryError}</span>
				<button class="underline cursor-pointer" onclick={loadDictionary}>Retry</button>
			</div>
		{/if}
	</div>
	{#if searchError}
		<p role="alert" class="text-danger text-sm">{searchError}</p>
	{/if}
	<div class="bg-surface my-1 w-full max-w-3xl h-px" aria-hidden="true"></div>
	{#if isSearching || itemData}
		{#if isSearching}
			<div>Loading...</div>
		{:else if itemData}
			<InfoCard
				{itemData}
				catalogItem={catalog?.[itemData.gameRef]}
				{relatedItems}
				onSelectItem={handleValueChange}
			/>
			{#if catalogError}
				<p class="text-muted-foreground text-sm">
					Extra item details are unavailable. Restart while online to refresh them.
				</p>
			{/if}
			<Statistics slug={itemData.slug} />
			<Orders
				slug={itemData.slug}
				itemName={itemData.i18n?.en.name}
				bulkTradable={itemData.bulkTradable ?? false}
			/>
		{/if}
	{:else}
		<section class="flex flex-col gap-3 w-full max-w-3xl" aria-labelledby="most-traded-heading">
			<div class="flex justify-between items-baseline gap-4">
				<h2 id="most-traded-heading">Most traded items</h2>
				<span class="text-muted-foreground text-xs">Sorted by liquidity</span>
			</div>
			{#if isLoadingMostTraded}
				<p role="status" class="text-muted-foreground text-sm">Loading most traded items...</p>
			{:else if mostTradedError}
				<div role="alert" class="flex items-center gap-2 text-sm">
					<span>Could not load the most traded items.</span>
					<Button onclick={loadMostTraded}>Retry</Button>
				</div>
			{:else if mostTradedItems.length === 0}
				<p class="text-muted-foreground text-sm">No liquidity data is available yet.</p>
			{:else}
				<!-- Tag filtering is temporarily disabled.
				{#if isLoadingDictionary}
					<p role="status" class="text-muted-foreground text-sm">Loading tag filters...</p>
				{:else if !dictionaryError && tagFilters.length > 0}
					<RadioGroup.Root
						aria-label="Filter most traded items by tag"
						class="flex flex-wrap gap-2"
						loop
						bind:value={selectedTag}
						onValueChange={() => visibleItemCount = 20}
					>
						<RadioGroup.Item
							value="__all__"
							class="data-[state=checked]:bg-accent hover:bg-surface px-3 py-1.5 border data-[state=checked]:border-accent focus-visible:outline-2 text-sm data-[state=checked]:text-accent-foreground cursor-pointer"
						>All</RadioGroup.Item>
						{#each tagFilters as { tag, liquidity } (tag)}
							<RadioGroup.Item
								value={tag}
								title={`Total liquidity: ${number.format(liquidity)}`}
								class="data-[state=checked]:bg-accent hover:bg-surface px-3 py-1.5 border data-[state=checked]:border-accent focus-visible:outline-2 text-sm capitalize data-[state=checked]:text-accent-foreground cursor-pointer"
							>{tag.replaceAll('_', ' ')}</RadioGroup.Item>
						{/each}
					</RadioGroup.Root>
				{/if}
				-->
				<ol class="divide-y divide-surface">
					{#each visibleMostTradedItems as item, index (item.slug)}
						<li>
							<button
								type="button"
								class="flex items-center gap-3 hover:bg-surface focus-visible:bg-surface px-3 py-2.5 focus-visible:outline-accent w-full text-sm text-left transition cursor-pointer"
								onclick={() => handleValueChange(item.slug)}
							>
								<span class="w-5 tabular-nums text-muted-foreground shrink-0">{index + 1}</span>
								<span class="flex-1">{item.name}</span>
								<span class="tabular-nums text-muted-foreground">
									{number.format(item.liquidity)}
								</span>
							</button>
						</li>
					{/each}
				</ol>
				{#if visibleItemCount < filteredMostTradedItems.length}
					<Button class="self-center" onclick={() => (visibleItemCount += 20)}>Show more</Button>
				{/if}
			{/if}
		</section>
	{/if}
</div>
