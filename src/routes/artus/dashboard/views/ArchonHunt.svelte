<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(isCurrent(world.archonHunt, now) ? (world.archonHunt.missions ?? []).map((mission, index) => ({
		title: `${index + 1}. ${mission.node}`, description: [mission.type, mission.faction].join(' · '),
		details: [`Level ${mission.minEnemyLevel}–${mission.maxEnemyLevel}`, mission.description].filter(Boolean), expiry: world.archonHunt.expiry,
	})) : []);
</script>

<ViewPanel title="Archon Hunt" {rows} {now} summary={world.archonHunt?.boss} />

