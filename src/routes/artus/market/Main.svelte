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

		fetch(`/api/v1/${slug}/get-item`)
			.then((res) => res.json())
			.then(({ data }: z.infer<typeof GetItemResponseSchema>) => {
				itemData = data;
			})
			.catch((err) => {
				console.error('Search failed:', err);
			})
			.finally(() => {
				isSearching = false;
			});
	};

	// Helper to normalize either an array or a keyed record into a record so we can safely index by string keys.
	const asRecord = <T,>(v: T[] | Partial<Record<string, T[]>>): Record<string, T[]> => {
		if (Array.isArray(v)) return { '0': v as T[] };
		return v as Record<string, T[]>;
	};

	let selectedGroup = $state('');

	let dataRange = $state('chart-30days');

	const loadOrdersData = async (slug: string | undefined) => {
		if (!slug) return;

		return fetch(`/api/v1/${slug}/get-orders`)
			.then((res) => res.json())
			.then(({ data }: z.infer<typeof GetOrdersResponseSchema>) => {
				return data;
			})
			.catch((err) => {
				console.error('Failed to load offers:', err);
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
