<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let trader = $derived(world.vaultTrader);
	// The shared trader model labels every premium price as ducats; omit those misleading units here.
	let rows: ViewRow[] = $derived(trader ? [
		...(isCurrent(trader, now) ? (trader.inventory ?? []).map((item) => ({ title: item.item, description: 'Current offering', expiry: trader.expiry })) : []),
		...(trader.schedule ?? []).filter((item) => isCurrent(item, now)).map((item) => ({ title: item.item, description: 'Featured rotation', expiry: item.expiry })),
	] : []);
</script>

<ViewPanel title="Prime Resurgence" {rows} {now} summary={trader?.location} />
