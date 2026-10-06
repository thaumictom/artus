<script lang="ts">
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import { oracleBounties } from '$lib/oracle-bounties.svelte';
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';

	let { world, now, localNow }: DashboardViewProps = $props();
	const categories = [
		{ value: 'Solaris United', label: 'Solaris United' },
		{ value: 'Ostrons', label: 'Ostrons' },
		{ value: 'Entrati', label: 'Entrati' },
		{ value: 'ZarimanSyndicate', label: 'Holdfasts / Zariman' },
		{ value: 'EntratiLabSyndicate', label: 'Cavia' },
		{ value: 'HexSyndicate', label: 'Hex' },
	] as const;
	let category = $state<string>(categories[0].value);
	let isOracleCategory = $derived(['ZarimanSyndicate', 'EntratiLabSyndicate', 'HexSyndicate'].includes(category));
	let selectedLabel = $derived(categories.find((item) => item.value === category)?.label ?? category);
	let snapshot = $derived(oracleBounties.snapshot);
	let currentOracle = $derived(snapshot?.expiry != null && snapshot.expiry > localNow);
	let rows: ViewRow[] = $derived(isOracleCategory
		? currentOracle ? (snapshot?.bounties[category] ?? []).map((bounty, index) => ({
			title: `${index + 1}. ${bounty.node}`,
			description: [bounty.missionType, bounty.faction, bounty.ally].filter(Boolean).join(' · '),
			details: [bounty.challenge, bounty.objective].filter(Boolean),
		})) : []
		: (world.syndicateMissions ?? [])
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

{#if isOracleCategory && oracleBounties.error}
	<p role="status" class="mb-4 text-danger text-sm">
		Could not refresh current bounties. Automatic retries are limited to once every five minutes.
	</p>
{/if}
<ViewPanel title="Bounties" countLabel="bounties" {rows} now={isOracleCategory ? localNow : now}
	expiry={isOracleCategory && currentOracle && snapshot?.expiry ? new Date(snapshot.expiry) : undefined}
	empty={isOracleCategory && oracleBounties.loading
		? 'Loading current bounties…' : `No current bounties for ${selectedLabel} in this snapshot.`}
>
	{#snippet headerAction()}
		<RadioGroup label="Bounty faction" variant="segmented" options={categories} bind:value={category} class="max-w-full" />
	{/snippet}
</ViewPanel>
