<script lang="ts">
	import CycleWidgets from '../widgets/CycleWidgets.svelte';
	import type { DashboardViewProps } from './view-types';

	let { world, now }: DashboardViewProps = $props();
</script>

<CycleWidgets {world} {now} />
