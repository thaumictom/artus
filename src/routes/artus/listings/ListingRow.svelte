<script lang="ts">
	import Icon from '@iconify/svelte';
	import ActionPopover from '$lib/components/ActionPopover.svelte';
	import type { InventoryItem } from '$lib/inventory';
	import { listingName, listingOwned, listingSlug } from './listing-utils';
	import type { Listing, ListingItem } from './types';

	let {
		order,
		itemDetails,
		inventoryItems,
		busy,
		menuOpenFor = $bindable<string | null>(null),
		onEdit,
		onVisibility,
		onSoldOne,
		onDelete,
		onOpenMarket,
	}: {
		order: Listing;
		itemDetails: Record<string, ListingItem>;
		inventoryItems: InventoryItem[];
		busy: boolean;
		menuOpenFor: string | null;
		onEdit: (order: Listing) => void;
		onVisibility: (order: Listing) => void;
		onSoldOne: (order: Listing, removeInventory: boolean) => void;
		onDelete: (order: Listing) => void;
		onOpenMarket: (slug: string) => void;
	} = $props();

	const name = $derived(listingName(order, itemDetails));
	const owned = $derived(listingOwned(order, itemDetails, inventoryItems));
	const menuItemClass = 'flex w-full cursor-pointer items-center gap-2 px-2 py-1.5 text-left text-sm hover:bg-elevated disabled:opacity-40';
</script>

<tr class="border-t border-border-secondary transition-colors hover:bg-surface/70">
	<td class="min-w-64 px-3 py-3.5">
		<div class="font-semibold text-foreground">{name}</div>
		{#if order.rank != null || order.subtype}
			<div class="mt-0.5 text-xs text-muted-foreground">
				{order.rank != null ? `Rank ${order.rank}` : ''}{order.rank != null && order.subtype ? ' · ' : ''}{order.subtype ?? ''}
			</div>
		{/if}
	</td>
	<td class="px-3 py-3.5">
		<span class={`inline-flex rounded-full border px-3 py-1 text-xs font-semibold ${order.type === 'sell' ? 'border-sky-500/30 bg-sky-500/15 text-sky-300' : 'border-violet-500/30 bg-violet-500/15 text-violet-300'}`}>
			{order.type === 'sell' ? 'Sell' : 'Buy'}
		</span>
	</td>
	<td class="px-3 py-3.5 text-right font-semibold tabular-nums">
		<span class="inline-flex items-center justify-end gap-1.5">{order.platinum}<img src="/icons/platinum.png" class="size-3.5" alt="platinum" /></span>
	</td>
	<td class="px-3 py-3.5 text-right font-semibold tabular-nums">{order.quantity}</td>
	<td class="px-3 py-3.5">
		<span class={`inline-flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-semibold ${order.visible ? 'border-accent/30 bg-accent/10 text-accent' : 'border-amber-500/30 bg-amber-500/10 text-amber-400'}`}>
			<span class={`size-1.5 rounded-full ${order.visible ? 'bg-accent' : 'bg-amber-400'}`}></span>{order.visible ? 'Visible' : 'Hidden'}
		</span>
	</td>
	<td class="px-3 py-3.5 text-right">
		<ActionPopover
			bind:open={() => menuOpenFor === order.id, (value) => { menuOpenFor = value ? order.id : null; }}
			triggerAriaLabel={`Actions for ${name}`}
			triggerClass="inline-flex items-center justify-center hover:bg-elevated border border-border-secondary size-7"
			contentClass="w-60"
		>
			{#snippet trigger()}<Icon icon="lucide:ellipsis" class="size-4" />{/snippet}
			<button type="button" disabled={busy} class={menuItemClass} onclick={() => { menuOpenFor = null; onEdit(order); }}><Icon icon="lucide:pencil" class="size-4" /> Edit listing</button>
			<button type="button" class={menuItemClass} onclick={() => { menuOpenFor = null; onOpenMarket(listingSlug(order, itemDetails)); }}><Icon icon="lucide:store" class="size-4" /> View market</button>
			<button type="button" disabled={busy} class={menuItemClass} onclick={() => onVisibility(order)}><Icon icon={order.visible ? 'lucide:eye-off' : 'lucide:eye'} class="size-4" /> {order.visible ? 'Hide listing' : 'Unhide listing'}</button>
			{#if order.type === 'sell'}
				<button type="button" disabled={busy || (order.perTrade ?? 1) !== 1} title={(order.perTrade ?? 1) !== 1 ? 'This listing must be sold in larger trade units' : undefined} class={menuItemClass} onclick={() => onSoldOne(order, false)}><Icon icon="lucide:check" class="size-4" /> Sold 1</button>
				<button type="button" disabled={busy || (order.perTrade ?? 1) !== 1 || owned < 1} title={owned < 1 ? 'No matching item in inventory' : undefined} class={menuItemClass} onclick={() => onSoldOne(order, true)}><Icon icon="lucide:package-minus" class="size-4" /> Sold 1 &amp; adjust inventory</button>
			{/if}
			<button type="button" disabled={busy} class={`${menuItemClass} text-danger hover:bg-danger/15`} onclick={() => onDelete(order)}><Icon icon="lucide:trash-2" class="size-4" /> Delete listing</button>
		</ActionPopover>
	</td>
</tr>
