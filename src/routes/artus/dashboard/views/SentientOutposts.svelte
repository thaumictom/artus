<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(world.sentientOutposts?.mission && isCurrent(world.sentientOutposts, now) ? [{
			title: world.sentientOutposts.mission.node,
			description: [world.sentientOutposts.mission.type, world.sentientOutposts.mission.faction].join(' · '),
			expiry: world.sentientOutposts.expiry,
		}] : []);
</script>

<ViewPanel title="Sentient Outposts" {rows} {now} />

