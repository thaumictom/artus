<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import ViewToolbar from '$lib/components/ViewToolbar.svelte';
	import WorldStateMissionCard from '$lib/components/WorldStateMissionCard.svelte';
	import {
		isAlertCompleted,
		loadAlertCompletions,
		setAlertsCompleted,
	} from '$lib/alert-completions.svelte';
	import { isCurrent, type DashboardViewProps } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let loaded = $state(false);
	let saving = $state(false);
	let error = $state('');
	let alerts = $derived((world.alerts ?? []).filter((item) => isCurrent(item, now)));
	let remainingAlerts = $derived(alerts.filter(({ id }) => !isAlertCompleted(id)));
	let uncheckedIds = $derived(remainingAlerts.flatMap(({ id }) => (id ? [id] : [])));
	let stats = $derived([
		{ value: remainingAlerts.length, label: 'remaining' },
		{ value: alerts.length - remainingAlerts.length, label: 'completed' },
	]);

	onMount(() => {
		// Prune only when entering Alerts, using all API alerts, including expired ones.
		void loadAlertCompletions((world.alerts ?? []).flatMap(({ id }) => (id ? [id] : [])))
			.then(() => {
				loaded = true;
			})
			.catch((cause) => {
				console.error('Could not load alert completions:', cause);
				error = 'Could not load completed alerts. Reopen Alerts to try again.';
			});
	});

	async function toggleCompleted(ids: string[], completed: boolean) {
		if (!loaded || saving || ids.length === 0) return;
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

<section class="flex flex-col gap-4 min-w-0" aria-label="Alerts">
	<ViewToolbar {stats} loading={!loaded} loadingText="Loading completion status…">
		{#snippet actions()}
			<Button
				variant="primary"
				class="inline-flex items-center gap-1.5 text-base"
				disabled={!loaded || saving || uncheckedIds.length === 0}
				onclick={() => toggleCompleted(uncheckedIds, true)}
			>
				<Icon icon="lucide:check" class="size-4" /> Complete all
			</Button>
		{/snippet}
	</ViewToolbar>
	{#if error}<p role="alert" class="text-danger text-sm">{error}</p>{/if}
	<ul class="flex flex-col gap-3">
		{#each alerts as alert (alert)}
			{@const completed = isAlertCompleted(alert.id)}
			{@const mission = alert.mission}
			<WorldStateMissionCard
				node={mission.node}
				reward={mission.reward}
				{completed}
				expiry={alert.expiry}
				{now}
				disabled={!loaded || saving || !alert.id}
				onCompletedChange={(checked) => {
					if (alert.id) void toggleCompleted([alert.id], checked);
				}}
			>
				{#snippet details()}
					{mission.type} · {mission.faction} ·
					<span class="tabular-nums whitespace-nowrap">
						Level {mission.minEnemyLevel}–{mission.maxEnemyLevel}
					</span>
				{/snippet}
			</WorldStateMissionCard>
		{:else}
			<li class="p-6 border border-surface text-muted-foreground text-sm text-center">
				No active alerts in this snapshot.
			</li>
		{/each}
	</ul>
</section>
