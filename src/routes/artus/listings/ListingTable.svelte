<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import Table, { type TableColumn } from '$lib/components/Table.svelte';
	import ActionPopover from '$lib/components/ActionPopover.svelte';
	import type { InventoryItem } from '$lib/inventory';
	import { listingItem, listingName, listingOwned, listingSlug } from './listing-utils';
	import type { Listing, ListingItem } from './types';

	let {
		orders,
		itemDetails,
		inventoryItems,
		busy,
		menuOpenFor = $bindable<string | null>(null),
		sortColumn,
		sortDirection,
		onSort,
		onEdit,
		onVisibility,
		onSoldOne,
		onDelete,
		onOpenMarket,
	}: {
		orders: Listing[];
		itemDetails: Record<string, ListingItem>;
		inventoryItems: InventoryItem[];
		busy: boolean;
		menuOpenFor: string | null;
		sortColumn: string;
		sortDirection: 'asc' | 'desc';
		onSort: (key: string) => void;
		onEdit: (order: Listing) => void;
		onVisibility: (order: Listing) => void;
		onSoldOne: (order: Listing, removeInventory: boolean) => void;
		onDelete: (order: Listing) => void;
		onOpenMarket: (slug: string) => void;
	} = $props();

	const columns: TableColumn[] = [
		{ key: 'name', label: 'Name', sortable: true, class: 'min-w-64' },
		{ key: 'type', label: 'Type', class: 'w-24' },
		{ key: 'price', label: 'Price', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'quantity', label: 'Quantity', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'status', label: 'Status', sortable: true, class: 'w-28' },
		{ key: 'actions', label: 'Actions', align: 'right', class: 'w-40' },
	];
</script>

<div class="overflow-hidden border border-border-secondary [&>div]:border-0">
	<Table {columns} {sortColumn} {sortDirection} {onSort} minWidth="900px">
		{#each orders as order (order.id)}
			<tr class="border-t border-border-secondary transition-colors hover:bg-surface/70">
				<td class="px-3 py-4 min-w-64">
					<div class="flex items-center gap-3">
						<div class="flex size-12 shrink-0 items-center justify-center overflow-hidden border border-border-secondary bg-background/70">
							{#if listingItem(order, itemDetails)?.icon}
								<img src={`https://warframe.market/static/assets/${listingItem(order, itemDetails)?.icon}`} alt="" loading="lazy" class="size-11 object-contain" />
							{:else}
								<Icon icon="lucide:package" class="size-5 text-muted-foreground" />
							{/if}
						</div>
						<div class="min-w-0">
							<div class="font-semibold text-foreground">{listingName(order, itemDetails)}</div>
							{#if order.rank != null || order.subtype}<div class="mt-0.5 text-muted-foreground text-xs">{order.rank != null ? `Rank ${order.rank}` : ''}{order.rank != null && order.subtype ? ' · ' : ''}{order.subtype ?? ''}</div>{/if}
						</div>
					</div>
				</td>
				<td class="px-3 py-4"><span class={`inline-flex rounded-full border px-3 py-1 text-xs font-semibold ${order.type === 'sell' ? 'border-sky-500/30 bg-sky-500/15 text-sky-300' : 'border-violet-500/30 bg-violet-500/15 text-violet-300'}`}>{order.type === 'sell' ? 'Sell' : 'Buy'}</span></td>
				<td class="px-3 py-4 text-right font-semibold tabular-nums"><span class="inline-flex items-center justify-end gap-1.5">{order.platinum}<img src="/icons/platinum.png" class="size-3.5" alt="platinum" /></span></td>
				<td class="px-3 py-4 text-right font-semibold tabular-nums">{order.quantity}</td>
				<td class="px-3 py-4"><span class={`inline-flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-semibold ${order.visible ? 'border-accent/30 bg-accent/10 text-accent' : 'border-amber-500/30 bg-amber-500/10 text-amber-400'}`}><span class={`size-1.5 rounded-full ${order.visible ? 'bg-accent' : 'bg-amber-400'}`}></span>{order.visible ? 'Visible' : 'Hidden'}</span></td>
				<td class="px-3 py-4 text-right">
					<div class="flex items-center justify-end gap-2">
						<Button disabled={busy} onclick={() => onEdit(order)} class="inline-flex h-9 items-center gap-1.5 px-3 text-xs"><Icon icon="lucide:pencil" class="size-3.5" /> Edit</Button>
						<ActionPopover bind:open={() => menuOpenFor === order.id, (value) => { menuOpenFor = value ? order.id : null; }} triggerAriaLabel={`Actions for ${listingName(order, itemDetails)}`} triggerClass="inline-flex items-center justify-center hover:bg-elevated border border-border-secondary size-9" contentClass="w-60">
							{#snippet trigger()}<Icon icon="lucide:ellipsis" class="size-4" />{/snippet}
							<button type="button" onclick={() => { menuOpenFor = null; onOpenMarket(listingSlug(order, itemDetails)); }} class="flex items-center gap-2 hover:bg-elevated px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:store" class="size-4" /> View market</button>
							<button type="button" disabled={busy} onclick={() => onVisibility(order)} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon={order.visible ? 'lucide:eye-off' : 'lucide:eye'} class="size-4" /> {order.visible ? 'Hide listing' : 'Unhide listing'}</button>
							{#if order.type === 'sell'}
								<button type="button" disabled={busy || (order.perTrade ?? 1) !== 1} title={(order.perTrade ?? 1) !== 1 ? 'This listing must be sold in larger trade units' : undefined} onclick={() => onSoldOne(order, false)} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:check" class="size-4" /> Sold 1</button>
								<button type="button" disabled={busy || (order.perTrade ?? 1) !== 1 || listingOwned(order, itemDetails, inventoryItems) < 1} title={listingOwned(order, itemDetails, inventoryItems) < 1 ? 'No matching item in inventory' : undefined} onclick={() => onSoldOne(order, true)} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:package-minus" class="size-4" /> Sold 1 and remove 1 from inventory</button>
							{/if}
							<button type="button" disabled={busy} onclick={() => onDelete(order)} class="flex items-center gap-2 text-danger hover:bg-danger/15 disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:trash-2" class="size-4" /> Delete listing</button>
						</ActionPopover>
					</div>
				</td>
			</tr>
		{/each}
	</Table>
</div>
