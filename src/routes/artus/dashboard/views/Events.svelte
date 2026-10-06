<script lang="ts">
	import ViewCard from '$lib/components/ViewCard.svelte';
	import ViewToolbar from '$lib/components/ViewToolbar.svelte';
	import WorldStateReward from '$lib/components/WorldStateReward.svelte';
	import Progress from '$lib/components/Progress.svelte';
	import { isCurrent, type DashboardViewProps } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let events = $derived((world.events ?? []).filter((item) => isCurrent(item, now)));
</script>

<section class="flex flex-col gap-4 min-w-0" aria-label="Events">
	<ViewToolbar stats={[{ value: events.length, label: 'active events' }]} />
	<ul class="flex flex-col gap-3">
		{#each events as event (event)}
			{@const location = [event.node, event.faction].filter(Boolean).join(' · ')}
			{@const hasScore = Number.isFinite(event.currentScore)
				&& Number.isFinite(event.maximumScore) && event.maximumScore > 0}
			{#snippet metadata()}
				<p class="font-medium text-sm break-words">{location}</p>
			{/snippet}
			<ViewCard {now} expiry={event.expiry} footer={location ? metadata : undefined}>
				<h3 class="font-semibold break-words">{event.description || 'Event'}</h3>
				{#if event.tooltip && event.tooltip !== event.description}
					<p class="mt-2 text-muted-foreground text-sm whitespace-pre-line break-words">{event.tooltip}</p>
				{/if}
				{#if event.rewards?.length}
					<div class="mt-4 pt-4 border-t border-surface">
						<p class="mb-2 text-muted-foreground text-xs uppercase tracking-widest">Rewards</p>
						<div class="flex flex-col gap-3">
							{#each event.rewards as reward}
								<WorldStateReward {reward} />
							{/each}
						</div>
					</div>
				{/if}
				{#snippet action()}
					{#if hasScore}
						<div class="flex flex-col gap-2 w-40 max-w-full">
							<span class="text-muted-foreground text-sm tabular-nums text-right">
								{event.currentScore.toLocaleString()} / {event.maximumScore.toLocaleString()}
							</span>
							<Progress value={event.currentScore} max={event.maximumScore}
								aria-label={`Progress for ${event.description || 'event'}`} />
						</div>
					{/if}
				{/snippet}
			</ViewCard>
		{:else}
			<li class="p-6 border border-surface text-muted-foreground text-sm text-center">
				No active events in this snapshot.
			</li>
		{/each}
	</ul>
</section>
