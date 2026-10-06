<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, validDate, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let activeArchimedeas = $derived((world.archimedeas ?? []).filter((item) => isCurrent(item, now)));
	let expiry = $derived(activeArchimedeas.map((item) => item.expiry).filter(validDate)
		.sort((a, b) => a.getTime() - b.getTime())[0]);
	let stats = $derived([
		{ value: activeArchimedeas.length, label: 'hunts' },
		{ value: activeArchimedeas.reduce((total, item) => total + item.missions.length, 0), label: 'missions' },
	]);
	let rows: ViewRow[] = $derived(activeArchimedeas.flatMap((item) => [
		{ title: item.type, details: item.personalModifiers.map((modifier) => `${modifier.name}: ${modifier.description}`) },
		...item.missions.map((mission, index) => ({
			title: `${index + 1}. ${mission.missionType}`, description: `${item.type} · ${mission.faction}`,
			details: [`${mission.deviation.name}: ${mission.deviation.description}`, ...mission.risks.map((risk) => `${risk.name}${risk.isHard ? ' (Elite)' : ''}: ${risk.description}`)],
		})),
	]));
</script>

<ViewPanel title="Archimedea" {stats} {rows} {now} {expiry} />
