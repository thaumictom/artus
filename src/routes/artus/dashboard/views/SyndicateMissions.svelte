<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.syndicateMissions ?? []).filter((item) => isCurrent(item, now)).flatMap((syndicate) => [
		...(syndicate.nodes ?? []).map((node) => ({ title: node, description: syndicate.syndicate, expiry: syndicate.expiry })),
		...(syndicate.jobs ?? []).map((job) => ({
			title: job.type || 'Bounty', description: syndicate.syndicate,
			details: [
				`Level ${job.enemyLevels.join('–')} · Mastery ${job.minMR}`,
				`Standing: ${job.standingStages.join(' / ')}`,
				...(job.rewardPool?.length ? [`Rewards: ${job.rewardPool.join(' · ')}`] : []),
			],
			expiry: syndicate.expiry,
		})),
	]));
</script>

<ViewPanel title="Syndicates &amp; Bounties" {rows} {now} />

