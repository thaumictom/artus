<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(world.simaris?.target ? [{
			title: world.simaris.target, description: world.simaris.isTargetActive ? 'Synthesis target active' : 'No active synthesis target',
		}] : []);
</script>

<ViewPanel title="Simaris" {rows} {now} />

