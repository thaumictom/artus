<script lang="ts">
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';

	let { world, now }: DashboardViewProps = $props();
	const categories = [
		{ value: 'Solaris United', label: 'Solaris United' },
		{ value: 'Ostrons', label: 'Ostrons' },
		{ value: 'Entrati', label: 'Entrati' },
	] as const;
	let category = $state<string>(categories[0].value);
	let rows: ViewRow[] = $derived((world.syndicateMissions ?? [])
		.filter((item) => item.syndicateKey === category && isCurrent(item, now))
		.flatMap((syndicate) => (syndicate.jobs ?? [])
			.filter((job) => isCurrent(job, now))
			.map((job) => {
				const levels = (job.enemyLevels ?? []).filter(Number.isFinite);
				return {
					title: job.type || 'Bounty',
					details: [
						[
							levels.length ? `Level ${levels.join('–')}` : undefined,
							Number.isFinite(job.minMR) ? `Mastery ${job.minMR}` : undefined,
						].filter(Boolean).join(' · '),
						...(job.standingStages?.length ? [`Standing: ${job.standingStages.join(' / ')}`] : []),
						...(job.rewardPool?.length ? [`Rewards: ${job.rewardPool.join(' · ')}`] : []),
					].filter(Boolean),
					expiry: job.expiry,
				};
			})));
</script>

<ViewPanel title="Bounties" countLabel="bounties" {rows} {now} empty={`No active bounties for ${category} in this snapshot.`}>
	{#snippet headerAction()}
		<RadioGroup label="Bounty faction" variant="segmented" options={categories} bind:value={category} class="max-w-full" />
	{/snippet}
</ViewPanel>
