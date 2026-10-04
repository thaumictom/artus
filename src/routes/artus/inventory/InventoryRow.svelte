<script module lang="ts">
	// Formatting is identical across rows; one formatter avoids native allocations per item/mount.
	const platinumFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });
</script>

<script lang="ts">
	import Icon from '@iconify/svelte';
	import { inventoryMarketSlug, type InventoryItem } from '$lib/inventory';
	import { marketAccount } from '$lib/market-account.svelte';
	import Button from '$lib/components/Button.svelte';
	import QuantityControl from '$lib/components/QuantityControl.svelte';
	import WarframeItem from '$lib/components/WarframeItem.svelte';
	import type { Listing } from '../listings/types';

	let {
		item,
		price,
		mastered,
		listing,
		listingsLoaded,
		isNew,
		onChangeQuantity,
		onOpenMarket,
		onOpenListing,
	}: {
		item: InventoryItem;
		price?: { median: number; from_current_offers: boolean };
		mastered: boolean;
		listing?: Listing;
		listingsLoaded: boolean;
		isNew: boolean;
		onChangeQuantity: (item: InventoryItem, delta: number) => void;
		onOpenMarket: (slug: string) => void;
		onOpenListing: (item: InventoryItem, listing: Listing | null) => void;
	} = $props();

	const wikiUrl = $derived(
		`https://wiki.warframe.com/w/Special:Search?search=${encodeURIComponent(item.name)}`,
	);
	const marketSlug = $derived(inventoryMarketSlug(item));
	function openListing() {
		if (!marketSlug) return;
		onOpenListing({ ...item, slug: marketSlug }, listing ?? null);
	}
</script>

<tr class="border-t border-border-secondary transition-colors hover:bg-surface/70">
	<td class="px-3 py-3.5 font-semibold text-foreground min-w-48">
		<WarframeItem item={marketSlug ?? ''} name={item.name}
			{mastered} hideOwned={true} listing={listing ?? null} tradable={!!marketSlug} {onOpenMarket}>
			{#snippet trailing()}
			{#if isNew}
				<span
					class="bg-accent rounded-full size-2 shrink-0"
					title="Newly added to inventory"
					aria-label="Newly added"
				></span>
			{/if}
			{/snippet}
		</WarframeItem>
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums">
		<QuantityControl value={item.quantity} label={item.name} onChange={(delta) => onChangeQuantity(item, delta)} />
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums">
		{#if price}
			<span class="inline-flex items-center justify-end gap-1" title={price.from_current_offers ? 'Current offer median; recent trades exist' : 'Recent trade median'}>
				{platinumFormatter.format(price.median)}
				<img src="/icons/platinum.png" class="size-3.5" alt="platinum" />
			</span>
		{:else}<span class="text-muted-foreground">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums">
		{#if item.ducats != null}
			<span class="inline-flex items-center justify-end gap-1">{item.ducats.toLocaleString()}<img src="/icons/ducats.png" class="size-3.5" alt="ducats" /></span>
		{:else}<span class="text-muted-foreground">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums font-semibold">
		{#if price}
			<span class="inline-flex items-center justify-end gap-1">{platinumFormatter.format(price.median * item.quantity)}<img src="/icons/platinum.png" class="size-3.5" alt="platinum" /></span>
		{:else}<span class="text-muted-foreground font-normal">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums font-semibold">
		{#if item.ducats != null}
			<span class="inline-flex items-center justify-end gap-1">{(item.ducats * item.quantity).toLocaleString()}<img src="/icons/ducats.png" class="size-3.5" alt="ducats" /></span>
		{:else}<span class="text-muted-foreground font-normal">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 text-right">
		<div class="flex justify-end items-center gap-1.5">
			{#if marketSlug}
				<Button
					size="icon"
					class="inline-flex justify-center items-center size-8"
					disabled={!marketAccount.session || !listingsLoaded}
					title={!marketAccount.session ? 'Log in to warframe.market to manage listings' : !listingsLoaded ? 'Loading listings' : listing ? 'Edit listing' : 'Create listing'}
					aria-label={`${listing ? 'Edit' : 'Create'} listing for ${item.name}`}
					onclick={openListing}
				>
					<Icon icon={listing ? 'lucide:pen-line' : 'lucide:plus'} class="size-4" />
				</Button>
			{/if}
			<Button
				size="icon"
				class="inline-flex justify-center items-center size-8"
				href={wikiUrl}
				target="_blank"
				rel="noopener noreferrer"
				title="View wiki"
				aria-label={`View ${item.name} on the wiki`}
			>
				<Icon icon="lucide:book-open" class="size-4" />
			</Button>
		</div>
	</td>
</tr>
