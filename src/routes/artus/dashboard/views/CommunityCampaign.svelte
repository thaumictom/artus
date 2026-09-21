<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(world.questToConquerCancer ? [{
			title: 'Quest to Conquer Cancer',
			description: 'Community progress toward the next goal',
			value: `${world.questToConquerCancer.count.toLocaleString()} / ${world.questToConquerCancer.goal.toLocaleString()}`,
		}] : []);
</script>

<ViewPanel title="Community Campaign" {rows} {now} />
