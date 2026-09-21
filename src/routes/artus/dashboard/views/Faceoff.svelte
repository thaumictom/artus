<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, validDate, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(world.faceoffBonus ? [{
			title: isCurrent(world.faceoffBonus, now) ? 'Bonus active' : 'Next bonus',
			description: isCurrent(world.faceoffBonus, now) ? 'Faceoff bonus is currently available.' : validDate(world.faceoffBonus.next) ? world.faceoffBonus.next.toLocaleString() : 'Schedule unavailable',
			expiry: isCurrent(world.faceoffBonus, now) ? world.faceoffBonus.expiry : undefined,
		}] : []);
</script>

<ViewPanel title="Faceoff Bonus" {rows} {now} />

