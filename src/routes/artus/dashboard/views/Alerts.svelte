<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import Button from '$lib/components/Button.svelte';
	import WarframeItem from '$lib/components/WarframeItem.svelte';
	import { formatTimeLeft } from '$lib/date';
	import { alertCompletions, loadAlertCompletions, setAlertsCompleted } from '$lib/alert-completions.svelte';
	import { isCurrent, validDate, type DashboardViewProps } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let loaded = $state(false);
	let saving = $state(false);
	let error = $state('');
	let alerts = $derived((world.alerts ?? []).filter((item) => isCurrent(item, now)));
	let completedCount = $derived(alerts.filter(({ id }) =>
		id && alertCompletions.completedIds.includes(id),
	).length);
	let uncheckedIds = $derived(alerts.flatMap(({ id }) =>
		id && !alertCompletions.completedIds.includes(id) ? [id] : [],
	));

	onMount(() => {
		// Prune only when entering Alerts, using all API alerts, including expired ones.
		void loadAlertCompletions((world.alerts ?? []).flatMap(({ id }) => id ? [id] : []))
			.then(() => { loaded = true; })
			.catch((cause) => {
				console.error('Could not load alert completions:', cause);
				error = 'Could not load completed alerts. Reopen Alerts to try again.';
			});
	});

	async function toggleCompleted(ids: string[], completed: boolean) {
		saving = true;
		error = '';
		try {
			await setAlertsCompleted(ids, completed);
		} catch (cause) {
			console.error('Could not save alert completion:', cause);
			error = 'Could not save alert completion. Please try again.';
		} finally {
			saving = false;
		}
	}
</script>

{#snippet deadline(expiry: Date)}
	<span class="text-sm text-muted-foreground tabular-nums whitespace-nowrap">
		Ends in <time datetime={expiry.toISOString()} title={expiry.toLocaleString()}>{formatTimeLeft(expiry, now)}</time>
	</span>
{/snippet}

<section class="flex flex-col gap-4 min-w-0" aria-label="Alerts">
	<header class="flex flex-wrap justify-between items-center gap-4 w-full">
		<div class="flex flex-wrap items-center divide-border-secondary divide-x text-base tabular-nums" aria-live="polite">
			{#if loaded}
				<div class="pr-4">{alerts.length - completedCount} remaining</div>
				<div class="pl-4 text-muted-foreground">{completedCount} completed</div>
			{:else}
				<p class="text-muted-foreground">Loading completion status…</p>
			{/if}
		</div>
		<div class="flex flex-wrap items-center gap-2">
			<Button variant="primary" class="inline-flex items-center gap-1.5 text-base"
				disabled={!loaded || saving || uncheckedIds.length === 0}
				onclick={() => toggleCompleted(uncheckedIds, true)}>
				<Icon icon="lucide:check" class="size-4" /> Complete all
			</Button>
		</div>
	</header>
	<div class="bg-surface my-1 w-full h-px"></div>
	{#if error}<p role="alert" class="text-danger text-sm">{error}</p>{/if}
	<ul class="flex flex-col gap-3">
		{#each alerts as alert}
			{@const completed = !!alert.id && alertCompletions.completedIds.includes(alert.id)}
			{@const mission = alert.mission}
			{@const reward = mission.reward}
			<li class="border transition-colors {completed ? 'bg-surface/30 border-surface' : 'bg-background border-border-secondary'}">
				<div class="flex items-start justify-between gap-4 p-4">
					<div class="min-w-0">
						<!-- countedItems already includes uncounted API items at quantity one. -->
						{#each reward?.countedItems ?? [] as item}
							<div class="mb-2 last:mb-0">
								<WarframeItem item={item.uniqueName} name={item.type} nameClass={completed ? 'font-medium text-muted-foreground' : 'font-semibold'}>
									{#snippet trailing()}
										{#if item.count > 1}<span class="text-sm text-muted-foreground tabular-nums">×{item.count.toLocaleString()}</span>{/if}
									{/snippet}
								</WarframeItem>
							</div>
						{/each}
						{#if reward && reward.credits > 0}
							<p class="mt-2 text-sm text-muted-foreground tabular-nums">{reward.credits.toLocaleString()} credits</p>
						{:else if !reward || !reward.countedItems.length}
							<p class="text-sm text-muted-foreground">Reward unavailable</p>
						{/if}
					</div>
					<label class="flex shrink-0 items-center gap-2 text-sm cursor-pointer {completed ? 'text-accent' : 'text-muted-foreground'}">
						<Checkbox
							checked={completed}
							disabled={!loaded || saving || !alert.id}
							aria-label={`Mark ${mission.node} as completed`}
							onCheckedChange={(checked) => { if (alert.id) void toggleCompleted([alert.id], checked); }}
						/>
						<span class="sr-only sm:not-sr-only">{completed ? 'Done' : 'Complete'}</span>
					</label>
				</div>
				<div class="flex flex-wrap items-center justify-between gap-x-4 gap-y-2 border-t border-surface bg-surface/20 px-4 py-3">
					<div class="min-w-0">
						<h3 class="font-medium text-sm break-words {completed ? 'text-muted-foreground' : ''}">{mission.node}</h3>
						<p class="mt-1 text-sm text-muted-foreground break-words">
							{mission.type} · {mission.faction} · <span class="whitespace-nowrap tabular-nums">Level {mission.minEnemyLevel}–{mission.maxEnemyLevel}</span>
						</p>
					</div>
					{#if validDate(alert.expiry)}{@render deadline(alert.expiry)}{/if}
				</div>
			</li>
		{:else}
			<li class="border border-surface p-6 text-sm text-muted-foreground text-center">No active alerts in this snapshot.</li>
		{/each}
	</ul>
</section>
