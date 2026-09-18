<script lang="ts">
	import Combobox from '$lib/components/Combobox.svelte';
	import { onMount } from 'svelte';
	import { z } from 'zod';
	import {
		ItemSchema,
		DictionarySchema,
		GetItemResponseSchema,
	} from '$lib/schemas';
	import Orders from './Orders.svelte';
	import Statistics from './Statistics.svelte';
	import InfoCard from './InfoCard.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { MarketCatalogSchema, type CatalogItem } from '$lib/market-catalog';

	let catalog = $state.raw<Record<string, CatalogItem> | null>(null);
	let catalogError = $state(false);

	let isLoadingDictionary = $state(false);
	let isSearching = $state(false);
	let dictionaryError = $state<string | null>(null);

	let dictionaryItems: { label: string; value: string; setSlug?: string | null; isSet: boolean }[] = $state([]);
	let selectedSlug = $state('');
	let searchError = $state<string | null>(null);
	let relatedItems = $derived.by(() => {
		const current = dictionaryItems.find((item) => item.value === itemData?.slug);
		const setSlug = current?.isSet ? current.value : current?.setSlug;
		if (!setSlug) return [];
		const setName = dictionaryItems.find((item) => item.value === setSlug)?.label.replace(/ Set$/, '');
		const order = (label: string) => label === 'Set' ? 0 : label === 'Blueprint' ? 1 : 2;
		return dictionaryItems
			.filter((item) => item.value === setSlug || item.setSlug === setSlug)
			.map((item) => ({
				value: item.value,
				label: item.value === setSlug ? 'Set'
					: setName && item.label.startsWith(`${setName} `)
						? item.label.slice(setName.length + 1)
						: item.label,
			}))
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
				setSlug: item.set_slug,
				isSet: item.tags.includes('set'),
			}));
		} catch (err) {
			console.error('Failed to load dictionary:', err);
			dictionaryError = 'Could not load the item list. Please try again.';
		} finally {
			isLoadingDictionary = false;
		}
	}

	onMount(() => {
		let disposed = false;
		void loadDictionary();
		void invoke('get_cached_market_items').then((response) => {
			if (!disposed) catalog = MarketCatalogSchema.parse(response);
		}).catch((error) => {
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

	const handleValueChange = (slug: string) => {
		if (!slug || isSearching || slug === itemData?.slug) return;
		isSearching = true;
		searchError = null;

		invoke('get_market_item', { slug })
			.then((response: any) => {
				const { data } = GetItemResponseSchema.parse(response);
				itemData = data;
				selectedSlug = data.slug;
			})
			.catch((err) => {
				console.error('Search failed:', err);
				searchError = 'Could not load this item. Please try again.';
			})
			.finally(() => {
				isSearching = false;
			});
	};
</script>

<div class="flex flex-col items-center gap-4 mx-auto p-8 w-full">
	<div class="flex flex-col gap-1 w-full max-w-2xl">
		<h1>View prices of any item on warframe.market</h1>
		<Combobox
			onValueChange={handleValueChange}
			type="single"
			items={dictionaryItems}
			bind:value={selectedSlug}
			disabled={isLoadingDictionary || isSearching}
			inputProps={{ placeholder: isLoadingDictionary ? 'Loading items...' : 'Search for an item...' }}
		></Combobox>
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
	{#if isSearching || itemData}
		<div class="bg-surface my-1 w-full max-w-2xl h-px"></div>
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
				<p class="text-sm text-muted-foreground">Extra item details are unavailable. Restart while online to refresh them.</p>
			{/if}
			<Statistics slug={itemData.slug} />
			<Orders
				slug={itemData.slug}
				itemName={itemData.i18n?.en.name}
				bulkTradable={itemData.bulkTradable ?? false}
			/>
		{/if}
	{/if}
</div>
