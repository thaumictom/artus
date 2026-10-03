<script lang="ts">
	import { formatTimeLeft } from '$lib/date';
	import Table from '$lib/components/Table.svelte';
	import type { TableColumn } from '$lib/components/table-types';
	import type { WorldState } from 'warframe-worldstate-parser';
	import { isBaroActive } from '../baro';
	import { validDate, type DashboardViewProps } from './view-types';

	type InventoryItem = WorldState['voidTrader']['inventory'][number];
	type SortColumn = 'item' | 'ducats' | 'credits';
	let { world, now }: DashboardViewProps = $props();
	let trader = $derived(world.voidTrader);
	let available = $derived(isBaroActive(trader, now));
	let sortColumn = $state<SortColumn>('item');
	let sortDirection = $state<'asc' | 'desc'>('asc');
	const columns: TableColumn[] = [
		{ key: 'item', label: 'Item', sortable: true },
		{ key: 'ducats', label: 'Ducats', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'credits', label: 'Credits', sortable: true, align: 'right', class: 'w-36' },
	];
	const formatter = new Intl.NumberFormat();
	let rows = $derived.by(() => {
		const inventory = available ? [...(trader.inventory ?? [])] : [];
		return inventory.sort((a, b) => {
			let difference: number;
			if (sortColumn === 'item') difference = a.item.localeCompare(b.item);
			else {
				const left = a[sortColumn];
				const right = b[sortColumn];
				// Missing costs stay at the end in either direction.
				if (!Number.isFinite(left)) return Number.isFinite(right) ? 1 : 0;
				if (!Number.isFinite(right)) return -1;
				difference = left - right;
			}
			return (sortDirection === 'asc' ? difference : -difference) || a.item.localeCompare(b.item);
		});
	});

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
		<td class="px-3 py-3.5 font-medium">{item.item}</td>
		<td class="px-3 py-3.5 tabular-nums text-right">
			<span class="inline-flex items-center justify-end gap-1.5">
				{Number.isFinite(item.ducats) ? formatter.format(item.ducats) : '—'}
				<img src="/icons/ducats.png" class="size-3.5" alt="ducats" />
			</span>
		</td>
		<td class="px-3 py-3.5 tabular-nums text-right">
			{Number.isFinite(item.credits) ? formatter.format(item.credits) : '—'}
		</td>
	</tr>
{/snippet}

<section class="flex flex-col gap-3 min-w-0" aria-label="Baro Ki'Teer">
	<header class="flex flex-wrap items-start justify-between gap-2">
		<div>
			<h2 class="font-medium">Baro Ki'Teer</h2>
			<p class="mt-1 text-muted-foreground text-sm">{trader?.location || 'Relay unavailable'}</p>
		</div>
		<div class="text-sm text-right">
			{#if available}
				<span class="inline-flex rounded-full border border-success/30 bg-success/15 px-3 py-1 text-xs font-semibold text-success">Active</span>
				<p class="mt-1 text-muted-foreground tabular-nums">
					Leaves in <time datetime={trader.expiry?.toISOString()}>{formatTimeLeft(trader.expiry, now)}</time>
					· {rows.length} items
				</p>
			{:else if validDate(trader?.activation) && trader.activation.getTime() > now}
				<p class="text-muted-foreground tabular-nums">
					Arrives in <time datetime={trader.activation.toISOString()}>{formatTimeLeft(trader.activation, now)}</time>
				</p>
			{:else}
				<p class="text-muted-foreground">Schedule updating</p>
			{/if}
		</div>
	</header>
	<Table
		{columns}
		{rows}
		rowKey={(item) => item.uniqueName}
		renderRow={inventoryRow}
		{sortColumn}
		{sortDirection}
		{onSort}
		minWidth="480px"
		emptyMessage={available ? 'Inventory is unavailable in this snapshot.' : 'Inventory will appear when Baro arrives and the dashboard refreshes.'}
	/>
</section>
