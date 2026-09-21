<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.archimedeas ?? []).filter((item) => isCurrent(item, now)).flatMap((item) => [
		{ title: item.type, details: item.personalModifiers.map((modifier) => `${modifier.name}: ${modifier.description}`), expiry: item.expiry },
		...item.missions.map((mission, index) => ({
			title: `${index + 1}. ${mission.missionType}`, description: `${item.type} · ${mission.faction}`,
			details: [`${mission.deviation.name}: ${mission.deviation.description}`, ...mission.risks.map((risk) => `${risk.name}${risk.isHard ? ' (Elite)' : ''}: ${risk.description}`)],
		})),
	]));
</script>

<ViewPanel title="Archimedea" {rows} {now} />

