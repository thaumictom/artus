<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { amount, isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(world.steelPath ? [
		...(world.steelPath.currentReward && isCurrent(world.steelPath, now) ? [{
				title: world.steelPath.currentReward.name, description: 'Current weekly offering',
				value: amount(world.steelPath.currentReward.cost, 'Steel Essence'), expiry: world.steelPath.expiry,
			}] : []),
		...(world.steelPath.evergreens ?? []).map((item) => ({ title: item.name, description: 'Permanent offering', value: amount(item.cost, 'Steel Essence') })),
		...(world.steelPath.rotation ?? []).map((item) => ({ title: item.name, description: 'Weekly rotation pool', value: amount(item.cost, 'Steel Essence') })),
	] : []);
</script>

<ViewPanel title="Steel Path Offerings" {rows} {now} />
