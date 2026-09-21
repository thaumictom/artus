<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, rewardText, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.alerts ?? []).filter((item) => isCurrent(item, now)).map(({ mission, expiry }) => ({
		title: mission.node, description: [mission.type, mission.faction, `Level ${mission.minEnemyLevel}–${mission.maxEnemyLevel}`].join(' · '),
		details: [mission.description, rewardText(mission.reward)].filter(Boolean), expiry,
	})));
</script>

<ViewPanel title="Alerts" {rows} {now} />

