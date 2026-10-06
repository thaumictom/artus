<script lang="ts">
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	const categories = [
		{ value: 'Steel Meridian', label: 'Steel Meridian' },
		{ value: 'Red Veil', label: 'Red Veil' },
		{ value: 'Cephalon Suda', label: 'Cephalon Suda' },
		{ value: 'Perrin Sequence', label: 'Perrin Sequence' },
		{ value: 'New Loka', label: 'New Loka' },
		{ value: 'Arbiters of Hexis', label: 'Arbiters of Hexis' },
	] as const;
	let category = $state<string>(categories[0].value);
	let rows: ViewRow[] = $derived((world.syndicateMissions ?? [])
		.filter((item) => item.syndicateKey === category && isCurrent(item, now))
		.flatMap((syndicate) => (syndicate.nodes ?? []).map((node) => ({
			title: node, expiry: syndicate.expiry,
		}))));
</script>

<ViewPanel title="Syndicates" countLabel="missions" {rows} {now} empty={`No active missions for ${category} in this snapshot.`}>
	{#snippet headerAction()}
		<RadioGroup label="Syndicate" variant="segmented" options={categories} bind:value={category} class="max-w-full" />
	{/snippet}
</ViewPanel>
