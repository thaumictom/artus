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

	let isLoadingDictionary = $state(false);
	let isSearching = $state(false);
	let dictionaryError = $state<string | null>(null);

	let dictionaryItems: { label: string; value: string }[] = $state([]);

	async function loadDictionary() {
		isLoadingDictionary = true;
		dictionaryError = null;
		try {
			const response = await invoke('get_market_dictionary');
			const data = DictionarySchema.parse(response);
			dictionaryItems = data.items.map((item) => ({
				label: item.name,
				value: item.slug,
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
	});

	let itemData: z.infer<typeof ItemSchema> | null = $state(null);

	const handleValueChange = (slug: string) => {
		if (!slug) return;
		isSearching = true;

		invoke('get_market_item', { slug })
			.then((response: any) => {
				const { data } = GetItemResponseSchema.parse(response);
				itemData = data;
			})
			.catch((err) => {
				console.error('Search failed:', err);
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
	{#if isSearching || itemData}
		<div class="bg-surface my-1 w-full max-w-2xl h-px"></div>
		{#if isSearching}
			<div>Loading...</div>
		{:else if itemData}
			<InfoCard {itemData} />
			<Statistics slug={itemData.slug} />
			<Orders
				slug={itemData.slug}
				itemName={itemData.i18n?.en.name}
				bulkTradable={itemData.bulkTradable ?? false}
			/>
		{/if}
	{/if}
</div>
