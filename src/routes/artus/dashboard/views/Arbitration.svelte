<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(world.arbitration?.node && world.arbitration.nodeKey !== 'SolNode000' && isCurrent(world.arbitration, now) ? [{
			title: world.arbitration.node, description: [world.arbitration.type, world.arbitration.enemy].filter(Boolean).join(' · '),
			details: world.arbitration.archwing ? ['Archwing required'] : [], expiry: world.arbitration.expiry,
		}] : []);
</script>

<ViewPanel title="Arbitration" {rows} {now} empty="Arbitration data is unavailable in this snapshot." />
