<script lang="ts">
	import Combobox from "$lib/components/Combobox.svelte";
	import { Axis, Spline, Bars, Chart, Layer, Tooltip } from "layerchart";
	import { scaleBand, scaleLinear } from "d3-scale";
	import { onMount } from "svelte";
	import { z } from "zod";
	import {
		ItemSchema,
		DictionarySchema,
		StatisticsSchema,
		ClosedStatisticItem,
	} from "$lib/schemas";

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

	let itemData: z.infer<typeof ItemSchema>["data"] | null = $state(null);

	const handleValueChange = (slug: string) => {
		if (!slug) return;
		isSearching = true;

		fetch(`/api/v1/${slug}/get-item`)
			.then((res) => res.json())
			.then(({ data }: z.infer<typeof ItemSchema>) => {
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

	let selectedGroup = $state<string>();
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
			{@const { icon, name } = itemData.i18n.en}
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
					{#if isGrouped && groups.length > 0}
						<select bind:value={selectedGroup}>
							{#each groups as group}
								<option value={group}>Group: {group}</option>
							{/each}
						</select>
					{/if}
					<Chart
						data={chartData}
						x="datetime"
						xScale={scaleBand().padding(0.4)}
						y="volume"
						yDomain={[0, null]}
						yNice
						y1="median"
						y1Range={({ yScale }) => yScale.domain()}
						height={300}
					>
						{#snippet children({ context })}
							<Layer>
								<Axis placement="left" rule />
								<Axis
									placement="right"
									scale={scaleLinear(
										context.y1Scale?.domain() ?? [],
										[context.height, 0],
									)}
									ticks={context.y1Scale?.ticks?.()}
									rule
								/>
								<Spline
									y={(d) => context.y1Scale?.(d.median)}
								/>
								<Spline
									y={(d) => context.y1Scale?.(d.moving_avg)}
								/>
								<Bars y="volume" class="fill-surface" />
							</Layer>
							<Tooltip.Root {context}>
								{#snippet children({ data })}
									<Tooltip.Header>{data.year}</Tooltip.Header>
									<Tooltip.List>
										<Tooltip.Item
											label="sales"
											value={data.sales}
											format="currencyRound"
										/>
										<Tooltip.Item
											label="efficiency"
											value={data.efficiency}
										/>
									</Tooltip.List>
								{/snippet}
							</Tooltip.Root>
						{/snippet}
					</Chart>
				{/if}
			{/await}
		{/if}
	{/if}
</div>
