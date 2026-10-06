<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(isCurrent(world.archonHunt, now) ? (world.archonHunt.missions ?? []).map((mission, index) => ({
		title: `${index + 1}. ${mission.node}`, description: [mission.type, mission.faction].filter(Boolean).join(' · '),
		details: [
			...(Number.isFinite(mission.minEnemyLevel) && Number.isFinite(mission.maxEnemyLevel)
				? [`Level ${mission.minEnemyLevel}–${mission.maxEnemyLevel}`] : []),
			...(mission.description ? [mission.description] : []),
		],
	})) : []);
</script>

<ViewPanel title="Archon Hunt" countLabel="missions" {rows} {now} headerSummary={world.archonHunt?.boss} expiry={world.archonHunt?.expiry} />
