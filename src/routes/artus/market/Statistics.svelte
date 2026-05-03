<script lang="ts">
	import { ClosedStatisticItem, StatisticsSchema } from '$lib/schemas';
	import { RadioGroup } from 'bits-ui';
	import Select from '$lib/components/Select.svelte';
	import Chart from './Chart.svelte';
	import type z from 'zod';
	import { invoke } from '@tauri-apps/api/core';

	let { slug }: { slug: string } = $props();

	const loadStatisticsData = async (slug: string | undefined) => {
		if (!slug) return null;

		try {
			const response = await invoke('get_market_statistics', { slug });
			const { payload }: z.infer<typeof StatisticsSchema> = StatisticsSchema.parse(response);

			const closed90Days = payload.statistics_closed['90days'];
			const closed48Hours = payload.statistics_closed['48hours'];

			if (!closed90Days.length) {
				return null;
			}

			groupByProperty = FilterProperties.find((prop) => prop in closed48Hours[0]);

			if (groupByProperty) {
				const grouped90Days = Object.groupBy(
					closed90Days,
					(item) => item[groupByProperty as keyof typeof item] as PropertyKey,
				);

				const grouped48Hours = Object.groupBy(
					closed48Hours,
					(item) => item[groupByProperty as keyof typeof item] as PropertyKey,
				);

				isGrouped = true;

				// Return group
				return {
					data90Days: grouped90Days,
					data48Hours: grouped48Hours,
				};
			}

			// Otherwise return un-grouped data
			return {
				data90Days: closed90Days,
				data48Hours: closed48Hours,
			};
		} catch (err) {
			console.error('Failed to load statistics:', err);
			return null;
		}
	};

	type StatItem = z.infer<typeof ClosedStatisticItem>;
	type StatData = StatItem[] | Partial<Record<PropertyKey, StatItem[]>>;

	const FilterProperties = ['mod_rank', 'rank', 'subtype'] as const;

	let dataRange = $state<'chart-90days' | 'chart-30days' | 'chart-48hours'>('chart-30days');

	let groupByProperty: (typeof FilterProperties)[number] | undefined = $state();
	let isGrouped = $state(false);

	let data90Days: StatData | undefined = $state();
	let data48Hours: StatData | undefined = $state();
	let selectedGroup: string | undefined = $state();
	let availableGroups = $derived(isGrouped && data90Days ? Object.keys(data90Days) : []);
	let chartData = $derived.by(() => {
		if (!data90Days || !data48Hours) return [];

		const getActiveData = (data: any) => {
			if (isGrouped) {
				return selectedGroup ? data[selectedGroup] || [] : [];
			}
			return data; // Return normal un-grouped data
		};

		const active90 = getActiveData(data90Days);
		const active48 = getActiveData(data48Hours);

		switch (dataRange) {
			case 'chart-90days':
				return active90;
			case 'chart-30days':
				return active90.slice(-30);
			case 'chart-48hours':
				return active48;
		}
	});

	$effect(() => {
		loadStatisticsData(slug).then((res) => {
			if (res) {
				data90Days = res.data90Days;
				data48Hours = res.data48Hours;

				// Set default selectedGroup
				if (isGrouped && !selectedGroup) {
					selectedGroup = Object.keys(res.data90Days)[0];
				}
			}
		});
	});
</script>

<div class="flex items-center gap-4 w-full max-w-2xl">
	{#if isGrouped}
		<Select
			type="single"
			value={selectedGroup}
			items={availableGroups.map((group) => ({
				label: `Group: ${group}`,
				value: String(group),
			}))}
			onValueChange={(value) => (selectedGroup = value)}
			triggerProps={{ class: 'max-w-none' }}
		></Select>
	{/if}
	<RadioGroup.Root class="flex select-none shrink-0" loop bind:value={dataRange}>
		<div class="p-1 border">
			<div
				class="inline-flex gap-1 *:data-[state=checked]:bg-surface *:px-4 *:py-1 *:data-[state=unchecked]:hover:text-foreground *:data-[state=unchecked]:text-muted-foreground *:data-[state=unchecked]:cursor-pointer"
			>
				<RadioGroup.Item
					id="chart-90days"
					value="chart-90days"
					class={{
						'striped-gradient cursor-not-allowed! *:text-muted-foreground! text-muted-foreground/30! opacity-30':
							Array.isArray(data90Days) && data90Days.length <= 30,
					}}
					disabled={Array.isArray(data90Days) && data90Days.length <= 30}
				>
					<span>90 days</span>
				</RadioGroup.Item>
				<RadioGroup.Item id="chart-30days" value="chart-30days">
					<span>30 days</span>
				</RadioGroup.Item>
				<RadioGroup.Item id="chart-48hours" value="chart-48hours" class="disabled:striped-gradient">
					<span>48 hours</span>
				</RadioGroup.Item>
			</div>
		</div>
	</RadioGroup.Root>
	{#if !isGrouped}
		<div class="bg-surface w-full h-px"></div>
	{/if}
</div>
<div class="w-full max-w-5xl">
	<Chart data={chartData} hourly={dataRange === 'chart-48hours'} />
</div>
