<script lang="ts">
	import { z } from 'zod';
	import { ClanReward } from 'warframe-worldstate-parser';
	import ViewPanel from './ViewPanel.svelte';
	import WarframeItem from '$lib/components/WarframeItem.svelte';
	import { amount, isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	const rewardSchema = z.object({
		RewardClaimed: z.boolean(),
		PointThreshold: z.number(),
		ItemCount: z.number(),
		Reward: z.string(),
	});
	let { world, now }: DashboardViewProps = $props();
	// The parser exposes raw rewards through the misspelled `rewwards` property.
	let rows: (ViewRow & { reward: ClanReward; endo?: number; relicPack: boolean })[] = $derived(
		isCurrent(world.clanWeeklyInitiative, now)
			? (world.clanWeeklyInitiative?.rewwards ?? []).flatMap((raw) => {
					const result = rewardSchema.safeParse(raw);
					if (!result.success) return [];
					const reward = new ClanReward(result.data, { locale: 'en' });
					// The API supplies bundle quantities; each uncommon bundle contains 50 Endo.
					const uncommonBundle =
						reward.uniqueName.split('/').at(-1)?.toLowerCase() === 'uncommonfusionbundle' ||
						reward.reward.toLowerCase() === 'uncommon fusion bundle';
					const relicPack = reward.uniqueName.split('/').at(-1)?.toLowerCase() === 'randomprojection'
						|| reward.reward.toLowerCase() === 'random projection';
					return [
						{
							title: reward.reward,
							reward,
							relicPack,
							endo: uncommonBundle ? reward.count * 50 : undefined,
							value: amount(reward.pointsRequired, 'points'),
						},
					];
				})
			: [],
	);
</script>

<ViewPanel
	title="Clan Initiative"
	stats={[]}
	{rows}
	{now}
	expiry={world.clanWeeklyInitiative?.expiry}
	headerSummary={world.clanWeeklyInitiative?.bonusRegion
		? `Bonus region: ${world.clanWeeklyInitiative.bonusRegion}`
		: undefined}
>
	{#snippet rowTitle(row)}
		<WarframeItem
			item={row.endo !== undefined
				? '/Lotus/Types/Items/MiscItems/FusionPoints'
				: row.relicPack ? '/Lotus/Types/BoosterPacks/RandomProjection' : row.reward.uniqueName}
			name={row.endo !== undefined ? 'Endo' : row.relicPack ? 'Relic Pack' : row.reward.reward}
		>
			{#snippet trailing()}
				{@const count = row.endo ?? row.reward.count}
				{#if count > 1}<span class="tabular-nums text-muted-foreground text-sm">
						×{count.toLocaleString()}
					</span>{/if}
			{/snippet}
		</WarframeItem>
	{/snippet}
</ViewPanel>
