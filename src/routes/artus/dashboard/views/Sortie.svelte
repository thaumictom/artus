<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(isCurrent(world.sortie, now) ? (world.sortie.variants ?? []).map((mission, index) => ({
		title: `${index + 1}. ${mission.node}`, description: `${mission.missionType} · ${world.sortie.faction}`,
		details: [mission.modifier, mission.modifierDescription].filter(Boolean), expiry: world.sortie.expiry,
	})) : []);
</script>

<ViewPanel title="Sortie" {rows} {now} summary={world.sortie?.boss} />

