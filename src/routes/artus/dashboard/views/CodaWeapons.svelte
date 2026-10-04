<script lang="ts">
	import WikiOfferings from './WikiOfferings.svelte';
	import type { DashboardViewProps } from './view-types';
	let { world, now }: DashboardViewProps = $props();
</script>

<WikiOfferings source="coda" {world} {now} />
