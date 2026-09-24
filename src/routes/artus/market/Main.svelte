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
	import { LazyStore } from '@tauri-apps/plugin-store';
	import { inventoryMarketSlug, inventoryNameKey, waitForInventorySave, type InventoryItem } from '$lib/inventory';
	import { mastery } from '$lib/mastery.svelte';
	import { MarketCatalogSchema, type CatalogItem } from '$lib/market-catalog';
	import MarketNotificationRules from './MarketNotificationRules.svelte';
	import { clearMarketNotificationTarget, marketNavigation } from '$lib/market-navigation.svelte';
	import { appNavigation, navigateTo } from '$lib/app-navigation.svelte';
	import { marketAccount } from '$lib/market-account.svelte';
	import Listings from './Listings.svelte';
	let activeMarketTab = $state<'browse' | 'listings'>('browse');

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
	let inventoryItems = $state<InventoryItem[]>([]);
	const ownedItems = $derived.by(() => {
		const bySlug = new Map<string, number>();
		const byName = new Map<string, number>();
		for (const item of inventoryItems) {
			if (item.isCustom || item.quantity <= 0) continue;
			const slug = inventoryMarketSlug(item);
			if (slug) bySlug.set(slug, (bySlug.get(slug) ?? 0) + item.quantity);
			else {
				const name = inventoryNameKey(item.name);
				byName.set(name, (byName.get(name) ?? 0) + item.quantity);
			}
		}
		return { bySlug, byName };
	});
	function ownedCountFor(slug: string) {
		const name = dictionaryItems.find((item) => item.value === slug)?.label;
		return (ownedItems.bySlug.get(slug) ?? 0) + (name ? ownedItems.byName.get(inventoryNameKey(name)) ?? 0 : 0);
	}
	const masteredSlugs = $derived.by(() => {
		const checked = new Set(mastery.checked);
		const slugs = new Set<string>();
		for (const item of mastery.items) {
			if (checked.has(item.key) && item.marketSlug) slugs.add(item.marketSlug);
			for (const component of item.components) {
				if ((checked.has(item.key) || checked.has(component.key)) && component.marketSlug)
					slugs.add(component.marketSlug);
			}
		}
		return slugs;
	});
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
				owned: ownedCountFor(item.value) > 0,
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
		void waitForInventorySave()
			.then(() => new LazyStore('inventory.json').get<InventoryItem[]>('items'))
			.then((items) => { if (!disposed) inventoryItems = items ?? []; })
			.catch((error) => console.error('Could not load inventory ownership:', error));
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
	let requestedSlug = '';

	function resetItem() {
		// Ignore pending detail requests after returning to the landing page.
		searchSequence += 1;
		requestedSlug = '';
		itemData = null;
		selectedSlug = '';
		searchError = null;
		isSearching = false;
		clearMarketNotificationTarget();
	}

	function closeItem() {
		navigateTo('market');
	}

	function loadItem(slug: string) {
		if (slug === requestedSlug) return;
		requestedSlug = slug;
		if (slug === itemData?.slug) {
			searchSequence += 1;
			selectedSlug = slug;
			isSearching = false;
			searchError = null;
			return;
		}
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
				requestedSlug = '';
				searchError = 'Could not load this item. Please try again.';
			})
			.finally(() => {
				if (!disposed && sequence === searchSequence) isSearching = false;
			});
	}

	const handleValueChange = (slug: string) => {
		if (slug === appNavigation.current.marketSlug) {
			if (searchError) loadItem(slug);
			return;
		}
		clearMarketNotificationTarget();
		navigateTo('market', slug);
	};

	$effect(() => {
		const location = appNavigation.current;
		if (location.section !== 'market') return;
		if (location.marketSlug) { activeMarketTab = 'browse'; loadItem(location.marketSlug); }
		else if (requestedSlug || itemData) resetItem();
	});
</script>

<div class="flex flex-col items-center gap-4 mx-auto p-8 w-full">
	<div class="flex gap-2 w-full max-w-5xl border-b border-border-secondary">
		<button class={`px-3 py-2 text-sm cursor-pointer ${activeMarketTab === 'browse' ? 'border-b-2 border-accent text-accent' : 'text-muted-foreground'}`} onclick={() => (activeMarketTab = 'browse')}>Browse</button>
		<button class={`px-3 py-2 text-sm ${activeMarketTab === 'listings' ? 'border-b-2 border-accent text-accent' : 'text-muted-foreground'} disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer`} disabled={!marketAccount.session} title={marketAccount.session ? 'Your warframe.market listings' : 'Log in to warframe.market to view listings'} onclick={() => (activeMarketTab = 'listings')}>Listings</button>
	</div>
	{#if activeMarketTab === 'listings' && marketAccount.session}
		<Listings />
	{:else}
	<div class="flex flex-col gap-1 w-full max-w-3xl">
		<div class="flex justify-between items-center gap-3">
			<h1>View prices of any item on warframe.market</h1>
		</div>
		<div class="flex gap-2 w-full h-full">
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
			<div>
				<div class="h-full">
					<MarketNotificationRules
						currentItem={itemData
							? {
									slug: itemData.slug,
									name: itemData.i18n?.en.name ?? itemData.slug,
								}
							: undefined}
					/>
				</div>
			</div>
			{#if itemData}
				<div class="shrink-0">
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
				mastered={masteredSlugs.has(itemData.slug)}
				ownedCount={ownedCountFor(itemData.slug)}
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
				highlightSince={marketNavigation.target?.slug === itemData.slug
					? marketNavigation.target.since
					: undefined}
				initialOrderType={marketNavigation.target?.slug === itemData.slug
					? marketNavigation.target.orderType
					: undefined}
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
	{/if}
</div>
