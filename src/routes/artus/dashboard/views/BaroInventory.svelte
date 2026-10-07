<script lang="ts">
	import Currency from '$lib/components/Currency.svelte';
	import { onMount } from 'svelte';
	import { formatTimeLeft } from '$lib/date';
	import Button from '$lib/components/Button.svelte';
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import Table from '$lib/components/Table.svelte';
	import WarframeItem from '$lib/components/WarframeItem.svelte';
	import {
		initializeWarframeItems,
		itemGameRef,
		warframeItems,
		refreshWarframeItemCatalog,
		resolveWarframeItem,
	} from '$lib/warframe-item.svelte';
	import type { TableColumn } from '$lib/components/table-types';
	import type { WorldState } from 'warframe-worldstate-parser';
	import { baroCategory, isBaroActive, type BaroCategory } from '../baro';
	import { validDate, type DashboardViewProps } from './view-types';

	type InventoryItem = WorldState['voidTrader']['inventory'][number];
	type SortColumn = 'item' | 'ducats' | 'credits';
	let { world, now }: DashboardViewProps = $props();
	onMount(initializeWarframeItems);
	let trader = $derived(world.voidTrader);
	let available = $derived(isBaroActive(trader, now));
	let sortColumn = $state<SortColumn>('item');
	let sortDirection = $state<'asc' | 'desc'>('asc');
	let categoryFilter = $state<'all' | BaroCategory>('all');
	const categoryOptions = [
		{ value: 'all', label: 'All' },
		{ value: 'mods', label: 'Mods' },
		{ value: 'weapons', label: 'Weapons' },
		{ value: 'misc', label: 'Misc' },
	] as const;
	let inventory = $derived(available ? (trader.inventory ?? []) : []);
	let filterOptions = $derived(
		categoryOptions.map((option) => ({
			...option,
			disabled: option.value !== 'all' && !warframeItems.catalogReady,
		})),
	);
	const columns: TableColumn[] = [
		{ key: 'item', label: 'Item', sortable: true },
		{ key: 'ducats', label: 'Ducats', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'credits', label: 'Credits', sortable: true, align: 'right', class: 'w-36' },
	];
	const nameFor = (item: InventoryItem) => resolveWarframeItem(item.uniqueName, item.item).name;
	let rows = $derived.by(() =>
		inventory
			.filter(
				(item) =>
					categoryFilter === 'all' ||
					baroCategory(warframeItems.catalog[itemGameRef(item.uniqueName)]) === categoryFilter,
			)
			.sort((a, b) => {
				let difference: number;
				if (sortColumn === 'item') difference = nameFor(a).localeCompare(nameFor(b));
				else {
					const left = a[sortColumn];
					const right = b[sortColumn];
					// Missing costs stay at the end in either direction.
					if (!Number.isFinite(left)) return Number.isFinite(right) ? 1 : 0;
					if (!Number.isFinite(right)) return -1;
					difference = left - right;
				}
				return (
					(sortDirection === 'asc' ? difference : -difference) ||
					nameFor(a).localeCompare(nameFor(b))
				);
			}),
	);

	function onSort(key: string) {
		if (key !== 'item' && key !== 'ducats' && key !== 'credits') return;
		if (sortColumn === key) sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		else {
			sortColumn = key;
			sortDirection = 'asc';
		}
	}
</script>

{#snippet inventoryRow(item: InventoryItem)}
	<tr class="hover:bg-surface/70 border-border-secondary border-t transition-colors">
		<td class="px-3 py-3.5">
			<WarframeItem item={item.uniqueName} name={item.item} />
		</td>
		<td class="px-3 py-3.5 tabular-nums text-right">
			<Currency class="inline-flex justify-end items-center gap-1.5" value={item.ducats} currency="ducats" />
		</td>
		<td class="px-3 py-3.5 tabular-nums text-right">
			<Currency value={item.credits} currency="credits" />
		</td>
	</tr>
{/snippet}

<section class="flex flex-col gap-3 min-w-0" aria-label="Baro Ki'Teer">
	<div class="flex justify-between gap-1">
		<div class="flex flex-col gap-1">
			<div class="font-semibold text-muted-foreground text-sm">Filter</div>
			<RadioGroup
				label="Filter Baro inventory category"
				options={filterOptions}
				bind:value={categoryFilter}
				class="max-w-full"
			/>
		</div>
		<div class="flex flex-col text-right">
			<p class="text-base">{trader?.location || 'Relay unavailable'}</p>
			{#if available || false}
				<p class="tabular-nums text-muted-foreground text-base">
					Leaves in <time datetime={trader.expiry?.toISOString()}>
						{formatTimeLeft(trader.expiry, now)}
					</time>
				</p>
			{:else if validDate(trader?.activation) && trader.activation.getTime() > now}
				<p class="tabular-nums text-muted-foreground">
					Arrives in <time datetime={trader.activation.toISOString()}>
						{formatTimeLeft(trader.activation, now)}
					</time>
				</p>
			{:else}
				<p class="text-muted-foreground text-base">Schedule updating</p>
			{/if}
		</div>
	</div>
	{#if warframeItems.catalogError}
		<p class="text-muted-foreground text-base" role="status">
			{warframeItems.catalogError}
			<Button variant="link" size="none" class="text-base" onclick={refreshWarframeItemCatalog}>
				Retry
			</Button>
		</p>
	{/if}
	<Table
		{columns}
		{rows}
		rowKey={(item) => item.uniqueName}
		renderRow={inventoryRow}
		{sortColumn}
		{sortDirection}
		{onSort}
		minWidth="480px"
		emptyMessage={!available
			? 'Inventory will appear when Baro arrives and the dashboard refreshes.'
			: inventory.length === 0
				? 'Inventory is unavailable in this snapshot.'
				: 'No items in this category.'}
	/>
</section>
