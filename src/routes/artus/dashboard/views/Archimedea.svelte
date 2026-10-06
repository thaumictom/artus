<script lang="ts">
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import { languageString } from 'warframe-worldstate-data/utilities';
	import { worldStateLabel } from '$lib/worldstate-labels';
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, validDate, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	const categories = [
		{ value: 'CT_LAB', label: worldStateLabel('archimedeaType', 'CT_LAB') },
		{ value: 'CT_HEX', label: worldStateLabel('archimedeaType', 'CT_HEX') },
	] as const;
	let category = $state<string>('CT_LAB');
	let selectedLabel = $derived(categories.find((item) => item.value === category)!.label);
	// The parser's typeKey is the English translation, rather than the raw CT_* tag.
	let activeArchimedeas = $derived((world.archimedeas ?? []).filter((item) =>
		item.typeKey === languageString(category, 'en') && isCurrent(item, now)));
	let expiry = $derived(activeArchimedeas.map((item) => item.expiry).filter(validDate)
		.sort((a, b) => a.getTime() - b.getTime())[0]);
	let rows: ViewRow[] = $derived(activeArchimedeas.flatMap((item) => [
		{ title: selectedLabel, details: item.personalModifiers.map((modifier) => `${worldStateLabel('archimedeaModifier', modifier.key, modifier.name)}: ${modifier.description}`) },
		...item.missions.map((mission, index) => ({
			title: `${index + 1}. ${mission.missionType}`, description: mission.faction,
			details: [`${worldStateLabel(category === 'CT_LAB' ? 'deepDeviation' : 'temporalDeviation', mission.deviation.key, mission.deviation.name)}: ${mission.deviation.description}`, ...mission.risks.map((risk) => `${worldStateLabel('archimedeaRisk', risk.key, risk.name)}${risk.isHard ? ' (Elite)' : ''}: ${risk.description}`)],
		})),
	]));
</script>

<ViewPanel title="Archimedea" stats={[]} {rows} {now} {expiry}
	empty={`No active ${selectedLabel} in this snapshot.`}>
	{#snippet headerAction()}
		<RadioGroup label="Archimedea hunt" variant="segmented" options={categories} bind:value={category} class="max-w-full" />
	{/snippet}
</ViewPanel>
