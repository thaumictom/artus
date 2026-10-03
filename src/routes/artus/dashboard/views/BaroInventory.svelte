<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { z } from 'zod';
	import { formatTimeLeft } from '$lib/date';
	import Button from '$lib/components/Button.svelte';
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import Table from '$lib/components/Table.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import type { TableColumn } from '$lib/components/table-types';
	import type { WorldState } from 'warframe-worldstate-parser';
	import { baroCategory, isBaroActive, type BaroCategory } from '../baro';
	import { validDate, type DashboardViewProps } from './view-types';

	type InventoryItem = WorldState['voidTrader']['inventory'][number];
	type SortColumn = 'item' | 'ducats' | 'credits';
	let { world, now }: DashboardViewProps = $props();
	let trader = $derived(world.voidTrader);
	let available = $derived(isBaroActive(trader, now));
	let sortColumn = $state<SortColumn>('item');
	let sortDirection = $state<'asc' | 'desc'>('asc');
	let categoryFilter = $state<'all' | BaroCategory>('all');
	const categoryOptions = [
		{ value: 'all', label: 'All' },
		{ value: 'mods', label: 'Mods' },
		{ value: 'weapons', label: 'Weapons' },
		{ value: 'appearance', label: 'Appearance' },
		{ value: 'misc', label: 'Misc' },
	] as const;
	const metadataSchema = z.object({
		category: z.string().nullish(),
		type: z.string().nullish(),
		description: z.string().nullish(),
		masteryReq: z.number().nullish(),
		rarity: z.string().nullish(),
		compatName: z.string().nullish(),
		tradable: z.boolean().optional(),
		levelStats: z.array(z.object({ stats: z.array(z.string()) })).nullish(),
	});
	type ItemMetadata = z.infer<typeof metadataSchema>;
	let metadata = $state<Record<string, ItemMetadata>>({});
	let catalogReady = $state(false);
	let catalogError = $state('');
	let catalogLoading = $state(false);
	let disposed = false;
	let catalogRequest = 0;
	let inventory = $derived(available ? (trader.inventory ?? []) : []);
	let filterOptions = $derived(categoryOptions.map((option) => ({
		...option,
		disabled: option.value !== 'all' && !catalogReady,
	})));
	const columns: TableColumn[] = [
		{ key: 'item', label: 'Item', sortable: true },
		{ key: 'ducats', label: 'Ducats', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'credits', label: 'Credits', sortable: true, align: 'right', class: 'w-36' },
	];
	const formatter = new Intl.NumberFormat();
	let rows = $derived.by(() => {
		return inventory.filter((item) => categoryFilter === 'all'
			|| baroCategory(metadataFor(item)) === categoryFilter).sort((a, b) => {
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

	function metadataFor(item: InventoryItem): ItemMetadata | undefined {
		// Baro lists store entries; the catalog keys identify their underlying game items.
		return metadata[item.uniqueName.replace('/StoreItems/', '/')];
	}

	function itemDetails(item: InventoryItem): string {
		const details = metadataFor(item);
		if (!details) return catalogReady ? 'Details unavailable.' : 'Loading details…';
		const summary = [
			details.type || details.category,
			details.rarity,
			details.masteryReq != null ? `MR ${details.masteryReq}` : undefined,
			details.compatName,
			details.tradable === true ? 'Tradeable' : undefined,
		].filter(Boolean);
		// Mods are most useful with their fully ranked effects; other items use their description.
		const effects = baroCategory(details) === 'mods' ? details.levelStats?.at(-1)?.stats.join(' · ') : undefined;
		const description = effects || details.description;
		if (description) summary.push(description.replace(/<[^>]*>/g, '').trim());
		return [...new Set(summary)].join(' · ') || 'Details unavailable.';
	}

	async function loadMetadata() {
		const request = ++catalogRequest;
		catalogLoading = true;
		try {
			const catalog = z.record(z.string(), z.unknown()).parse(await invoke('get_cached_market_items'));
			const next: Record<string, ItemMetadata> = {};
			for (const [uniqueName, value] of Object.entries(catalog)) {
				const parsed = metadataSchema.safeParse(value);
				if (parsed.success) next[uniqueName] = parsed.data;
			}
			if (disposed || request !== catalogRequest) return;
			metadata = next;
			catalogReady = true;
			catalogError = '';
		} catch (error) {
			if (disposed || request !== catalogRequest) return;
			catalogError = 'Item details could not be loaded.';
			console.error('Could not read Baro item metadata:', error);
		} finally {
			if (!disposed && request === catalogRequest) catalogLoading = false;
		}
	}

	onMount(() => {
		let unlisten: UnlistenFn | undefined;
		void listen('api_catalogs_fetched', () => { void loadMetadata(); })
			.then((stop) => {
				if (disposed) stop();
				else unlisten = stop;
			})
			.catch((error) => console.error('Could not listen for Baro catalog updates:', error));
		void loadMetadata();
		return () => {
			disposed = true;
			unlisten?.();
		};
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
		<td class="px-3 py-3.5">
			<Tooltip class="block w-full cursor-help text-left" align="start">
				{#snippet children()}
					<span class="block font-medium">{item.item}</span>
					<span class="block mt-1 text-muted-foreground text-xs line-clamp-2">{itemDetails(item)}</span>
				{/snippet}
				{#snippet content()}
					<p class="text-muted-foreground text-xs">Game reference / uniqueName</p>
					<p class="mt-1 font-mono text-xs break-all">{item.uniqueName}</p>
				{/snippet}
			</Tooltip>
		</td>
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
					· {inventory.length} items
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
	<div class="flex flex-wrap items-center justify-between gap-2">
		<RadioGroup label="Filter Baro inventory category" options={filterOptions} bind:value={categoryFilter} class="max-w-full" />
		{#if available && categoryFilter !== 'all'}
			<p class="text-muted-foreground text-xs">Showing {rows.length} of {inventory.length} items</p>
		{/if}
	</div>
	{#if catalogError}
		<p class="text-muted-foreground text-sm" role="status">
			{catalogError}
			<Button variant="link" size="none" class="text-sm" disabled={catalogLoading} onclick={loadMetadata}>Retry</Button>
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
		emptyMessage={!available ? 'Inventory will appear when Baro arrives and the dashboard refreshes.'
			: inventory.length === 0 ? 'Inventory is unavailable in this snapshot.' : 'No items in this category.'}
	/>
</section>
