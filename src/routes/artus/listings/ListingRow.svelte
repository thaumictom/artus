<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import WarframeItem from '$lib/components/WarframeItem.svelte';
	import type { InventoryItem } from '$lib/inventory';
	import { listingName, listingOwned, listingSlug } from './listing-utils';
	import type { Listing, ListingItem } from './types';

	let {
		order,
		virtualIndex,
		measureRow,
		itemDetails,
		inventoryItems,
		busy,
		median,
		onEdit,
		onVisibility,
		onSoldOne,
		onDelete,
		onOpenMarket,
	}: {
		order: Listing;
		virtualIndex: number;
		measureRow: (element: HTMLTableRowElement) => void;
		itemDetails: Record<string, ListingItem>;
		inventoryItems: InventoryItem[];
		busy: boolean;
		median?: { median: number; from_current_offers: boolean };
		onEdit: (order: Listing) => void;
		onVisibility: (order: Listing) => void;
		onSoldOne: (order: Listing) => void;
		onDelete: (order: Listing) => void;
		onOpenMarket: (slug: string) => void;
	} = $props();

	const name = $derived(listingName(order, itemDetails));
	const owned = $derived(listingOwned(order, itemDetails, inventoryItems));
	const priceFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });
	function handleRowClick(event: MouseEvent) {
		if (!(event.target as HTMLElement).closest('button, a')) onEdit(order);
	}
</script>

<tr
	data-index={virtualIndex}
	use:measureRow
	class="hover:bg-surface/70 border-border-secondary border-t transition-colors cursor-pointer"
	onclick={handleRowClick}
	onkeydown={(event) => {
		if (event.target === event.currentTarget && (event.key === 'Enter' || event.key === ' ')) {
			event.preventDefault();
			onEdit(order);
		}
	}}
	tabindex="0"
	role="button"
	aria-label={`Edit ${name} listing`}
>
	<td class="px-3 py-3.5 min-w-64">
		<WarframeItem item={listingSlug(order, itemDetails)} {name} ownedCount={owned}
			listing={order.type === 'sell' ? order : null} hideListing={true} {onOpenMarket} />
		{#if order.rank != null || order.subtype}
			<p class="mt-1 text-sm text-muted-foreground">{[order.rank != null ? `Rank ${order.rank}` : '', order.subtype].filter(Boolean).join(' · ')}</p>
		{/if}
	</td>
	<td class="px-3 py-3.5">
		<span
			class={`inline-flex rounded-full border px-3 py-1 text-sm font-semibold ${order.type === 'sell' ? 'border-sky-500/30 bg-sky-500/15 text-sky-300' : 'border-violet-500/30 bg-violet-500/15 text-violet-300'}`}
		>
			{order.type === 'sell' ? 'Sell' : 'Buy'}
		</span>
	</td>
	<td class="px-3 py-3.5">
		<Button
			size="none"
			class={`inline-flex items-center gap-1.5 rounded-full border px-3 py-1 text-sm font-semibold whitespace-nowrap transition-colors focus-visible:outline-2 focus-visible:outline-accent ${order.visible ? 'border-success/30 bg-success/15 text-success hover:bg-success/25' : 'border-border bg-muted/15 text-muted-foreground hover:bg-muted/25'}`}
			disabled={busy}
			title={order.visible ? 'Hide listing' : 'Unhide listing'}
			aria-label={`Toggle visibility for ${name}`}
			aria-pressed={order.visible}
			onclick={() => onVisibility(order)}
		>
			<Icon icon={order.visible ? 'lucide:check' : 'lucide:x'} class="size-3.5" aria-hidden="true" />
			{order.visible ? 'Visible' : 'Hidden'}
		</Button>
	</td>
	<td class="px-3 py-3.5 tabular-nums text-right">
		{#if median && Number.isFinite(median.median)}
			<span
				class="inline-flex justify-end items-center gap-1"
				title={median.from_current_offers
					? 'Current offer median; recent trades exist'
					: 'Recent trade median'}
			>
				{priceFormatter.format(median.median)}
				<img src="/icons/platinum.png" class="size-3.5" alt="platinum" />
			</span>
		{:else}<span class="text-muted-foreground">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 font-semibold tabular-nums text-right">
		<span class="inline-flex justify-end items-center gap-1.5">
			{order.platinum}
			<img src="/icons/platinum.png" class="size-3.5" alt="platinum" />
		</span>
	</td>
	<td class="px-3 py-3.5 font-semibold tabular-nums text-right">{order.quantity}</td>
	<td class="px-3 py-3.5 text-right">
		<div class="flex justify-end items-center gap-1.5">
			{#if order.type === 'sell'}
				<Button
					size="icon"
					class="inline-flex justify-center items-center size-8"
					disabled={busy || (order.perTrade ?? 1) !== 1}
					title={(order.perTrade ?? 1) !== 1
						? 'This listing must be sold in larger trade units'
						: owned < 1
							? 'Sold 1'
							: 'Sold 1 & adjust inventory'}
					aria-label={owned < 1 ? 'Sold 1' : 'Sold 1 & adjust inventory'}
					onclick={() => onSoldOne(order)}
				>
					<Icon icon={owned < 1 ? 'lucide:check' : 'lucide:package-minus'} class="size-4" />
				</Button>
			{/if}
			<Button
				size="icon"
				class="inline-flex justify-center items-center hover:bg-danger/15 border-danger size-8 text-danger"
				disabled={busy}
				title="Delete listing"
				aria-label="Delete listing"
				onclick={() => onDelete(order)}
			>
				<Icon icon="lucide:trash-2" class="size-4" />
			</Button>
		</div>
	</td>
</tr>
