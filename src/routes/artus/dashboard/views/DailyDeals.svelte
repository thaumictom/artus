<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, amount, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.dailyDeals ?? []).filter((item) => isCurrent(item, now)).map((item) => ({
		title: item.item, value: amount(item.salePrice, 'platinum'),
		description: `${item.discount}% off · Normally ${amount(item.originalPrice, 'platinum')}`,
		details: [`${Math.max(0, item.total - item.sold).toLocaleString()} of ${item.total.toLocaleString()} remaining`], expiry: item.expiry,
	})));
</script>

<ViewPanel title="Daily Deals" {rows} {now} />

