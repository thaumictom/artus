<script lang="ts">
	import { formatTimeLeft } from '$lib/date';
	import ViewPanel from './ViewPanel.svelte';
	import { amount, isCurrent, validDate, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let trader = $derived(world.voidTrader);
	let available = $derived(Boolean(trader && !trader.completed && validDate(trader.activation) && validDate(trader.expiry) && isCurrent(trader, now)));
	let summary = $derived([
		trader?.location || 'Relay unavailable',
		available ? `Leaves in ${formatTimeLeft(trader.expiry, now)}`
			: validDate(trader?.activation) && trader.activation.getTime() > now
				? `Arrives in ${formatTimeLeft(trader.activation, now)}` : 'Schedule updating',
	].join(' · '));
	let rows: ViewRow[] = $derived(available ? (trader.inventory ?? []).map((item) => ({
		title: item.item, value: amount(item.ducats, 'ducats'), description: amount(item.credits, 'credits'),
	})) : []);
</script>

<ViewPanel title="Baro Ki'Teer" {rows} {now} {summary} empty={available ? 'Inventory is unavailable in this snapshot.' : 'Inventory will appear when Baro arrives and the dashboard refreshes.'} />
