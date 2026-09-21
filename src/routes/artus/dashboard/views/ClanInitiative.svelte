<script lang="ts">
	import { z } from 'zod';
	import { ClanReward } from 'warframe-worldstate-parser';
	import ViewPanel from './ViewPanel.svelte';
	import { amount, isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	const rewardSchema = z.object({ RewardClaimed: z.boolean(), PointThreshold: z.number(), ItemCount: z.number(), Reward: z.string() });
	let { world, now }: DashboardViewProps = $props();
	// The parser exposes raw rewards through the misspelled `rewwards` property.
	let rows: ViewRow[] = $derived(isCurrent(world.clanWeeklyInitiative, now) ? (world.clanWeeklyInitiative?.rewwards ?? []).flatMap((raw) => {
		const result = rewardSchema.safeParse(raw);
		if (!result.success)
			return [];
		const reward = new ClanReward(result.data, { locale: 'en' });
		return [{ title: reward.rewardString(), value: amount(reward.pointsRequired, 'points'), expiry: world.clanWeeklyInitiative?.expiry }];
	}) : []);
</script>

<ViewPanel title="Clan Initiative" {rows} {now} summary={world.clanWeeklyInitiative ? `Bonus region: ${world.clanWeeklyInitiative.bonusRegion}` : undefined} />
