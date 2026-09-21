<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.persistentEnemies ?? []).map((item) => ({
		title: item.agentType, description: item.isDiscovered ? item.lastDiscoveredAt : 'Not currently located',
		details: [`Region: ${item.region} · Rank ${item.rank}`],
		value: Number.isFinite(item.healthPercent) ? `${item.healthPercent.toFixed(1)}% health` : undefined,
	})));
</script>

<ViewPanel title="Persistent Enemies" {rows} {now} />

