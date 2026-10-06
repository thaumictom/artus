<script lang="ts">
	import WarframeItem from '$lib/components/WarframeItem.svelte';
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, amount, type DashboardViewProps } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows = $derived((world.dailyDeals ?? []).filter((item) => isCurrent(item, now)).map((item) => ({
		title: item.item,
		reference: item.uniqueName,
		salePrice: Number.isFinite(item.salePrice) ? item.salePrice : undefined,
		originalPrice: Number.isFinite(item.originalPrice) ? item.originalPrice : undefined,
		discount: Number.isFinite(item.discount) && item.discount > 0 ? item.discount : undefined,
		details: [`${Math.max(0, item.total - item.sold).toLocaleString()} of ${item.total.toLocaleString()} remaining`], expiry: item.expiry,
	})));
</script>

<ViewPanel title="Daily Deals" {rows} {now}>
	{#snippet rowTitle(row)}
		<WarframeItem item={row.reference} name={row.title} />
	{/snippet}
	{#snippet rowAction(row)}
		<div class="flex flex-wrap items-center justify-end gap-x-3 gap-y-1 text-sm tabular-nums">
			{#if row.originalPrice !== undefined && row.salePrice !== undefined && row.originalPrice > row.salePrice}
				<del class="text-muted-foreground"><span class="sr-only">Original price: </span>{amount(row.originalPrice, 'platinum')}</del>
			{/if}
			<span class="font-semibold text-accent"><span class="sr-only">Sale price: </span>{amount(row.salePrice, 'platinum')}</span>
			{#if row.discount}<span class="text-muted-foreground">{row.discount}% off</span>{/if}
		</div>
	{/snippet}
</ViewPanel>
