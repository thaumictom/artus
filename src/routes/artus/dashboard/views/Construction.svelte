<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(world.constructionProgress ? [
		{ title: 'Fomorian', value: `${world.constructionProgress.fomorianProgress}%` },
		{ title: 'Razorback', value: `${world.constructionProgress.razorbackProgress}%` },
	] : []);
</script>

<ViewPanel title="Construction" {rows} {now} />

