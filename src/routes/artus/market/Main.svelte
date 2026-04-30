<script lang="ts">
	import Combobox from "$lib/components/Combobox.svelte";
	import { onMount } from "svelte";
	import { z } from "zod";
	import {
		ItemSchema,
		DictionarySchema,
		StatisticsSchema,
		ClosedStatisticItem,
		GetOrdersResponseSchema,
		GetItemResponseSchema,
	} from "$lib/schemas";
	import Select from "$lib/components/Select.svelte";
	import Chart from "./Chart.svelte";
	import Orders from "./Orders.svelte";
	import { Slider, Tabs } from "bits-ui";

	let isLoadingDictionary = $state(false);
	let isSearching = $state(false);

	let dictionaryItems: { label: string; value: string }[] = $state([]);

	onMount(() => {
		isLoadingDictionary = true;
		fetch("https://api.thaumictom.de/warframe/v1/dictionary.json")
			.then((res) => res.json())
			.then((data: z.infer<typeof DictionarySchema>) => {
				dictionaryItems = data.tradeable_items.map((item) => ({
					label: item.name,
					value: item.slug,
				}));
			})
			.catch((err) => {
				console.error("Failed to load dictionary:", err);
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
				console.error("Search failed:", err);
			})
			.finally(() => {
				isSearching = false;
			});
	};

	const autoGroupByUnknownKey = <T extends Record<string, any>>(
		data: T[],
		knownKeys: Set<string> = new Set(),
	): Partial<Record<string, T[]>> | T[] => {
		if (data.length === 0) return data;

		// Find the first key in the data that is NOT defined in your Zod schema
		const dynamicKey = Object.keys(data[0]).find(
			(key) => !knownKeys.has(key),
		);

		// If no dynamic key exists, assume datetime is unique and return the data as-is
		if (!dynamicKey) {
			return data;
		}

		// Group by the dynamically discovered key
		return Object.groupBy(data, (item) => String(item[dynamicKey]));
	};

	let selectedGroup = $state("0");

	const loadStatisticsData = async (slug: string) => {
		if (!slug) return;

		return fetch(`/api/v1/${slug}/get-statistics`)
			.then((res) => res.json())
			.then(({ payload }: z.infer<typeof StatisticsSchema>) => {
				const closed90Days = payload.statistics_closed["90days"];

				const knownKeys = new Set(
					Object.keys(ClosedStatisticItem.shape),
				);

				return autoGroupByUnknownKey(closed90Days, knownKeys);
			})
			.catch((err) => {
				console.error("Failed to load statistics:", err);
			});
	};

	const loadOrdersData = async (slug: string) => {
		if (!slug) return;

		return fetch(`/api/v1/${slug}/get-orders`)
			.then((res) => res.json())
			.then(({ data }: z.infer<typeof GetOrdersResponseSchema>) => {
				return data;
			})
			.catch((err) => {
				console.error("Failed to load offers:", err);
			});
	};
</script>

<div class="flex flex-col gap-8 mx-auto p-8 w-full max-w-2xl">
	<div class="flex flex-col gap-1 w-full">
		<h1>View prices of any item on warframe.market</h1>
		<Combobox
			onValueChange={handleValueChange}
			type="single"
			items={dictionaryItems}
			disabled={isLoadingDictionary || isSearching}
			inputProps={{ placeholder: "Search for an item..." }}
		></Combobox>
	</div>
	{#if isSearching || itemData}
		<div class="h-px bg-surface my-4"></div>
		{#if isSearching}
			<div>Loading...</div>
		{:else if itemData}
			{@const { icon, name } = itemData.i18n!.en}
			<div class="border p-4 flex gap-4 items-center">
				<img
					src={`https://warframe.market/static/assets/${icon}`}
					alt={name}
					class="size-16 aspect-square bg-surface text-transparent"
				/>
				<div>
					<h1 class="font-medium">{name}</h1>
				</div>
			</div>
			{#await loadStatisticsData(itemData.slug)}
				<div>Loading...</div>
			{:then statisticsData}
				{#if statisticsData}
					{@const isGrouped = !Array.isArray(statisticsData)}
					{@const groups = isGrouped
						? Object.keys(statisticsData)
						: []}
					{@const chartData = Array.isArray(statisticsData)
						? statisticsData
						: (statisticsData[selectedGroup ?? groups[0]] ?? [])}
					<div class="flex items-center gap-4 mb-4">
						{#if isGrouped && groups.length > 0}
							<Select
								type="single"
								bind:value={selectedGroup}
								items={groups.map((group) => ({
									label: group,
									value: group,
								}))}
							></Select>
						{/if}
						<Tabs.Root value="30days">
							<Tabs.List class="flex border-b">
								<Tabs.Trigger
									value="90days"
									class="px-4 py-2 data-[state=active]:border-b-2 data-[state=active]:border-primary"
								>
									90 days
								</Tabs.Trigger>
								<Tabs.Trigger
									value="30days"
									class="px-4 py-2 data-[state=active]:border-b-2 data-[state=active]:border-primary"
								>
									30 days
								</Tabs.Trigger>
								<Tabs.Trigger
									value="48hours"
									class="px-4 py-2 data-[state=active]:border-b-2 data-[state=active]:border-primary"
								>
									48 hours
								</Tabs.Trigger>
							</Tabs.List>
						</Tabs.Root>
					</div>
					<Chart data={chartData} />
				{/if}
			{/await}
			{#await loadOrdersData(itemData.slug)}
				<div>Loading orders...</div>
			{:then orderData}
				{#if orderData}
					<Orders {orderData} />
				{/if}
			{/await}
		{/if}
	{/if}
</div>
