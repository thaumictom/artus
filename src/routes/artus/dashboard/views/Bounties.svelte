<script lang="ts">
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import { formatTimeLeft } from '$lib/date';
	import { oracleBounties } from '$lib/oracle-bounties.svelte';
	import { languageString } from 'warframe-worldstate-data/utilities';
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
	let bountySource = $state('hub');
	const sources = [{ value: 'hub', label: 'Hub bounties' }, { value: 'field', label: 'Field bounties' }];
	const fieldTags: Record<string, 'CetusSyndicate' | 'SolarisSyndicate' | 'EntratiSyndicate'> = { Ostrons: 'CetusSyndicate', 'Solaris United': 'SolarisSyndicate', Entrati: 'EntratiSyndicate' };
	let field = $derived(oracleBounties.snapshot?.fieldBounties);
	let currentField = $derived(field && field.expiry > localNow ? field : null);
	let fieldRows: ViewRow[] = $derived(currentField && fieldTags[category]
		? Object.entries(currentField[fieldTags[category]]).map(([location, jobs]) => ({
			title: location.replace(/^Bounty/, '').replace(/([a-z])([A-Z])/g, '$1 $2'),
			details: jobs.map((job) => languageString(job, 'en')),
		})) : []);
	let isOracleCategory = $derived(['ZarimanSyndicate', 'EntratiLabSyndicate', 'HexSyndicate'].includes(category));
	let showField = $derived(!isOracleCategory && bountySource === 'field');
	let selectedLabel = $derived(categories.find((item) => item.value === category)?.label ?? category);
	let snapshot = $derived(oracleBounties.snapshot);
	let currentOracle = $derived(snapshot?.expiry != null && snapshot.expiry > localNow);
	let expiry = $derived(showField && currentField ? new Date(currentField.expiry)
		: isOracleCategory && currentOracle && snapshot?.expiry ? new Date(snapshot.expiry) : undefined);
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

{#if (isOracleCategory || showField) && oracleBounties.error}
	<p role="status" class="mb-4 text-danger text-sm">
		Could not refresh current bounties. Automatic retries are limited to once every five minutes.
	</p>
{/if}
<ViewPanel title="Bounties" rows={showField ? fieldRows : rows} now={isOracleCategory || showField ? localNow : now}
	empty={(isOracleCategory || showField) && oracleBounties.loading
		? 'Loading current bounties…' : `No current bounties for ${selectedLabel} in this snapshot.`}
>
	{#snippet header()}
		<div class="@container/filters flex flex-col gap-2">
			<div class="flex flex-wrap @max-[52rem]/filters:justify-between items-center gap-x-8 gap-y-4">
				<div class="flex flex-col gap-1">
					<div class="font-semibold text-muted-foreground text-sm">Faction</div>
					<RadioGroup label="Bounty faction" variant="segmented" options={categories} bind:value={category} class="max-w-full" />
				</div>
				{#if !isOracleCategory}
					<div class="flex flex-col gap-1">
						<div class="font-semibold text-muted-foreground text-sm">Bounty source</div>
						<RadioGroup label="Bounty source" variant="segmented" options={sources} bind:value={bountySource} />
					</div>
				{/if}
				{#if expiry}
					<span class="ml-auto text-sm text-muted-foreground tabular-nums whitespace-nowrap">
						Ends in <time datetime={expiry.toISOString()} title={expiry.toLocaleString()}>{formatTimeLeft(expiry, localNow)}</time>
					</span>
				{/if}
			</div>
		</div>
	{/snippet}
</ViewPanel>
