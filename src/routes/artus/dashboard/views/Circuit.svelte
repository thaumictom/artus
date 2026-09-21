<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.duviriCycle?.choices ?? []).map((choice) => ({
		title: choice.category, details: choice.choices,
	})));
</script>

<ViewPanel title="The Circuit" {rows} {now} />

