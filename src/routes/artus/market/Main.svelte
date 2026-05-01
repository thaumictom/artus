<script lang="ts">
	import Combobox from '$lib/components/Combobox.svelte';
	import { onMount } from 'svelte';
	import { z } from 'zod';
	import {
		ItemSchema,
		DictionarySchema,
		StatisticsSchema,
		ClosedStatisticItem,
		GetOrdersResponseSchema,
		GetItemResponseSchema,
	} from '$lib/schemas';
	import Select from '$lib/components/Select.svelte';
	import Chart from './Chart.svelte';
	import Orders from './Orders.svelte';
	import { Label, RadioGroup } from 'bits-ui';
	import Statistics from './Statistics.svelte';
	import InfoCard from './InfoCard.svelte';
	import { invoke } from '@tauri-apps/api/core';

	let isLoadingDictionary = $state(false);
	let isSearching = $state(false);

	let dictionaryItems: { label: string; value: string }[] = $state([]);

	onMount(() => {
		isLoadingDictionary = true;
		fetch('https://api.thaumictom.de/warframe/v1/dictionary.json')
			.then((res) => res.json())
			.then((data: z.infer<typeof DictionarySchema>) => {
				dictionaryItems = data.tradeable_items.map((item) => ({
					label: item.name,
					value: item.slug,
				}));
			})
			.catch((err) => {
				console.error('Failed to load dictionary:', err);
			})
			.finally(() => {
				isLoadingDictionary = false;
			});
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
			inputProps={{ placeholder: 'Search for an item...' }}
		></Combobox>
	</div>
	{#if isSearching || itemData}
		<div class="bg-surface my-1 w-full max-w-2xl h-px"></div>
		{#if isSearching}
			<div>Loading...</div>
		{:else if itemData}
			<InfoCard {itemData} />
			<Statistics slug={itemData.slug} />
			<Orders slug={itemData.slug} itemName={itemData.i18n?.en.name} />
		{/if}
	{/if}
</div>
