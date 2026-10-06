<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import {
		clanInitiativeCompletions, getClanInitiativeRewards, initializeClanInitiativeCompletions,
		isClanInitiativeRewardCompleted, setClanInitiativeRewardsCompleted,
	} from '$lib/clan-initiative-completions.svelte';
	import ViewPanel from './ViewPanel.svelte';
	import WarframeItem from '$lib/components/WarframeItem.svelte';
	import { amount, isCurrent, type DashboardViewProps } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let saving = $state(false);
	let error = $state('');
	let rows = $derived(
		isCurrent(world.clanWeeklyInitiative, now)
			? getClanInitiativeRewards(world.clanWeeklyInitiative).flatMap(({ id, reward }) => {
					// The API supplies bundle quantities; each uncommon bundle contains 50 Endo.
					const uncommonBundle =
						reward.uniqueName.split('/').at(-1)?.toLowerCase() === 'uncommonfusionbundle' ||
						reward.reward.toLowerCase() === 'uncommon fusion bundle';
					const relicPack = reward.uniqueName.split('/').at(-1)?.toLowerCase() === 'randomprojection'
						|| reward.reward.toLowerCase() === 'random projection';
					return [
						{
							id,
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
	let uncheckedIds = $derived(rows.filter((row) => !isClanInitiativeRewardCompleted(row.id)).map((row) => row.id));
	onMount(() => {
		void initializeClanInitiativeCompletions().catch((cause) => {
			console.error('Could not load clan initiative completions:', cause);
			error = 'Could not load completion status. Reopen Clan Initiative to try again.';
		});
	});
	async function toggleCompleted(ids: string[], completed: boolean) {
		if (!clanInitiativeCompletions.loaded || saving || !ids.length) return;
		saving = true;
		error = '';
		try {
			await setClanInitiativeRewardsCompleted(ids, completed);
		} catch (cause) {
			console.error('Could not save clan initiative completion:', cause);
			error = 'Could not save completion status. Please try again.';
		} finally {
			saving = false;
		}
	}
</script>

<ViewPanel
	title="Clan Initiative"
	stats={[]}
	{rows}
	{now}
	rowCompleted={(row) => isClanInitiativeRewardCompleted(row.id)}
	onRowCompletedChange={(row, checked) => void toggleCompleted([row.id], checked)}
	completionDisabled={!clanInitiativeCompletions.loaded || saving}
	expiry={world.clanWeeklyInitiative?.expiry}
	headerSummary={world.clanWeeklyInitiative?.bonusRegion
		? `Bonus region: ${world.clanWeeklyInitiative.bonusRegion}`
		: undefined}
>
	{#snippet headerAction()}
		<Button variant="primary" class="inline-flex items-center gap-1.5 text-base"
			disabled={!clanInitiativeCompletions.loaded || saving || !uncheckedIds.length}
			onclick={() => toggleCompleted(uncheckedIds, true)}>
			<Icon icon="lucide:check" class="size-4" /> Complete all
		</Button>
	{/snippet}
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
{#if error}<p role="alert" class="mt-3 text-danger text-sm">{error}</p>{/if}
