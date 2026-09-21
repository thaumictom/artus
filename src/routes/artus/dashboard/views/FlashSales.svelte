<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.flashSales ?? []).filter((item) => item.isShownInMarket && isCurrent(item, now)).map((item) => ({
		title: item.item,
		description: [item.isFeatured ? 'Featured' : '', item.isPopular ? 'Popular' : '', item.discount > 0 ? `${item.discount}% off` : ''].filter(Boolean).join(' · '),
		expiry: item.expiry,
	})));
</script>

<ViewPanel title="Featured Market Items" {rows} {now} />

