<script lang="ts">
	import ViewToolbar from '$lib/components/ViewToolbar.svelte';
	import Progress from '$lib/components/Progress.svelte';
	import WorldStateMissionCard from '$lib/components/WorldStateMissionCard.svelte';
	import WorldStateReward from '$lib/components/WorldStateReward.svelte';
	import { hasInvasionReward, invasionRewardOpportunities } from '$lib/invasion-rewards';
	import { factionColor } from '$lib/faction-colors';
	import type { DashboardViewProps } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let invasions = $derived((world.invasions ?? []).filter((item) => !item.completed));
	let opportunities = $derived(invasions.flatMap(invasionRewardOpportunities));

	let stats = $derived([
		{ value: invasions.length, label: 'active' },
		{ value: opportunities.length, label: 'notable rewards' },
	]);
</script>

<section class="flex flex-col gap-4 min-w-0" aria-label="Invasions">
	<ViewToolbar {stats} />
	<ul class="flex flex-col gap-3">
		{#each invasions as invasion (invasion)}
			{@const percentage = Number.isFinite(invasion.completion)
				? Math.min(100, Math.max(0, invasion.completion))
				: null}
			{@const twoRewards =
				hasInvasionReward(invasion.attacker.reward) && hasInvasionReward(invasion.defender.reward)}
			<WorldStateMissionCard node={invasion.node} {now}>
				<div class="gap-4 grid grid-cols-1 sm:grid-cols-2">
					{#each [invasion.attacker, invasion.defender].sort((a, b) => Number(hasInvasionReward(b.reward)) - Number(hasInvasionReward(a.reward))) as side, index}
						<div
							class="min-w-0 {index === 1
								? 'border-t border-border-secondary pt-4 sm:border-t-0 sm:border-l sm:pt-0 sm:pl-4'
								: ''}"
						>
							<p class="mb-2 font-medium text-muted-foreground text-xs uppercase tracking-widest">
								{side.faction}
							</p>
							{#if side.reward}
								<WorldStateReward reward={side.reward} />
							{:else}
								<p class="text-muted-foreground text-sm">
									{invasion.vsInfestation && side.factionKey === 'Infested'
										? '—'
										: 'Reward unavailable'}
								</p>
							{/if}
						</div>
					{/each}
				</div>
				{#snippet footerAside()}
					<div class="flex flex-col gap-2 ml-auto w-52 max-w-full shrink-0">
						<div
							class="flex flex-wrap justify-between gap-x-4 gap-y-1 tabular-nums text-muted-foreground text-sm"
						>
							{#if percentage === null}
								<span>Progress unavailable</span>
							{:else if invasion.vsInfestation}
								<span>Infested remaining</span>
								<span>{percentage.toFixed(1)}%</span>
							{:else}
								<span>{invasion.attacker.faction} {percentage.toFixed(1)}%</span>
								<span>{invasion.defender.faction} {(100 - percentage).toFixed(1)}%</span>
							{/if}
						</div>
						<Progress
							value={percentage}
							trackColor={twoRewards ? factionColor(invasion.defender.factionKey) : undefined}
							gap={twoRewards ? 3 : 0}
							indicatorColor={twoRewards ? factionColor(invasion.attacker.factionKey) : undefined}
							aria-label={`${invasion.vsInfestation ? 'Infested remaining' : 'Attacker control'} at ${invasion.node}`}
							aria-valuetext={percentage === null
								? 'Progress unavailable'
								: invasion.vsInfestation
									? `${percentage.toFixed(1)}% Infested remaining`
									: `${invasion.attacker.faction} ${percentage.toFixed(1)}%, ${invasion.defender.faction} ${(100 - percentage).toFixed(1)}%`}
						/>
					</div>
				{/snippet}
				{#snippet details()}
					{#if invasion.desc}{invasion.desc}&nbsp;·&nbsp;
					{/if}{invasion.attacker.faction} vs. {invasion.defender.faction}
				{/snippet}
			</WorldStateMissionCard>
		{:else}
			<li class="p-6 border border-surface text-muted-foreground text-sm text-center">
				No active invasions in this snapshot.
			</li>
		{/each}
	</ul>
</section>
