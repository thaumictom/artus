<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import type { InventoryItem } from '$lib/inventory';
	import { listingName, listingOwned, listingSlug } from './listing-utils';
	import type { Listing, ListingItem } from './types';

	let {
		order,
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
	class="hover:bg-surface/70 border-t transition-colors cursor-pointer"
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
		<Button
			variant="link"
			size="none"
			class="flex items-center gap-1 font-semibold text-foreground text-left"
			onclick={() => onOpenMarket(listingSlug(order, itemDetails))}
		>
			{#if order.visible === false}
				<Icon icon="material-symbols:visibility-off-outline-rounded" class="mr-0.5 size-3.5" />
			{/if}
			{name}
			<Icon icon="material-symbols:arrow-outward-rounded" class="size-4" />
		</Button>
		{#if order.rank != null || order.subtype}
			<div class="mt-0.5 text-muted-foreground text-xs">
				{order.rank != null ? `Rank ${order.rank}` : ''}{order.rank != null && order.subtype
					? ' · '
					: ''}{order.subtype ?? ''}
			</div>
		{/if}
	</td>
	<td class="px-3 py-3.5">
		<span
			class={`inline-flex rounded-full border px-3 py-1 text-xs font-semibold ${order.type === 'sell' ? 'border-sky-500/30 bg-sky-500/15 text-sky-300' : 'border-violet-500/30 bg-violet-500/15 text-violet-300'}`}
		>
			{order.type === 'sell' ? 'Sell' : 'Buy'}
		</span>
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
			<Button
				size="icon"
				class="inline-flex justify-center items-center size-8"
				disabled={busy}
				title={order.visible ? 'Hide listing' : 'Unhide listing'}
				aria-label={order.visible ? 'Hide listing' : 'Unhide listing'}
				onclick={() => onVisibility(order)}
			>
				<Icon icon={order.visible ? 'lucide:eye-off' : 'lucide:eye'} class="size-4" />
			</Button>
			{#if order.type === 'sell'}
				<Button
					size="icon"
					class="inline-flex justify-center items-center size-8"
					disabled={busy || (order.perTrade ?? 1) !== 1 || owned < 1}
					title={(order.perTrade ?? 1) !== 1
						? 'This listing must be sold in larger trade units'
						: owned < 1
							? 'No matching item in inventory'
							: 'Sold 1 & adjust inventory'}
					aria-label="Sold 1 & adjust inventory"
					onclick={() => onSoldOne(order)}
				>
					<Icon icon="lucide:package-minus" class="size-4" />
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
