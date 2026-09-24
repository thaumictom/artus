<script lang="ts">
	import Icon from '@iconify/svelte';
	import { inventoryMarketSlug, type InventoryItem } from '$lib/inventory';

	let {
		item,
		onChangeQuantity,
		onOpenMarket,
	}: {
		item: InventoryItem;
		onChangeQuantity: (item: InventoryItem, delta: number) => void;
		onOpenMarket: (slug: string) => void;
	} = $props();

	const wikiUrl = $derived(
		`https://wiki.warframe.com/w/Special:Search?search=${encodeURIComponent(item.name)}`,
	);
	const marketSlug = $derived(inventoryMarketSlug(item));
	const platinumFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });
</script>

<tr class="border-t border-border-secondary transition-colors hover:bg-surface/70">
	<td class="px-3 py-3.5 font-semibold text-foreground min-w-48">
		<span class="break-words">{item.name}</span>
		{#if item.category}<div class="mt-0.5 text-muted-foreground text-xs font-normal">{item.category}</div>{/if}
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums">
		<div class="inline-flex items-center gap-2">
			<button
				class="flex items-center justify-center border border-border-secondary size-6 hover:bg-surface cursor-pointer"
				aria-label={`Remove one ${item.name}`}
				onclick={() => onChangeQuantity(item, -1)}
			><Icon icon="lucide:minus" class="size-3.5" /></button>
			<span class="min-w-6 text-center">{item.quantity}</span>
			<button
				class="flex items-center justify-center border border-border-secondary size-6 hover:bg-surface cursor-pointer"
				aria-label={`Add one ${item.name}`}
				onclick={() => onChangeQuantity(item, 1)}
			><Icon icon="lucide:plus" class="size-3.5" /></button>
		</div>
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums">
		{#if item.marketMedian != null}
			<span class="inline-flex items-center justify-end gap-1" title={item.marketMedianUsesOfferFallback ? 'Current offer median; no recent trade median' : 'Recent trade median'}>
				{platinumFormatter.format(item.marketMedian)}
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
		{#if item.marketMedian != null}
			<span class="inline-flex items-center justify-end gap-1">{platinumFormatter.format(item.marketMedian * item.quantity)}<img src="/icons/platinum.png" class="size-3.5" alt="platinum" /></span>
		{:else}<span class="text-muted-foreground font-normal">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums font-semibold">
		{#if item.ducats != null}
			<span class="inline-flex items-center justify-end gap-1">{(item.ducats * item.quantity).toLocaleString()}<img src="/icons/ducats.png" class="size-3.5" alt="ducats" /></span>
		{:else}<span class="text-muted-foreground font-normal">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 text-right">
		<div class="inline-flex items-center gap-3">
			{#if marketSlug}<button class="text-accent hover:underline cursor-pointer" onclick={() => onOpenMarket(marketSlug!)}>Market</button>{/if}
			<a class="text-muted-foreground hover:text-foreground hover:underline" href={wikiUrl} target="_blank" rel="noopener noreferrer">Wiki ↗</a>
		</div>
	</td>
</tr>
