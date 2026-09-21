<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { validDate, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(world.kinepage?.message ? [{
			title: 'Latest message', description: world.kinepage.message,
			details: validDate(world.kinepage.timestamp) ? [world.kinepage.timestamp.toLocaleString()] : [],
		}] : []);
</script>

<ViewPanel title="Kinepage" {rows} {now} />

