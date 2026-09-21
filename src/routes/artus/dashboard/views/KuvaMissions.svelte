<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.kuva ?? []).filter((item) => isCurrent(item, now)).map((item) => ({
		title: item.node, description: [item.type, item.enemy].filter(Boolean).join(' · '), expiry: item.expiry,
	})));
</script>

<ViewPanel title="Kuva Missions" {rows} {now} empty="Kuva mission data is unavailable in this snapshot." />

