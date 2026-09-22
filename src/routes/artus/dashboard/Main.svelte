<script lang="ts">
	import { onMount } from 'svelte';
	import { dashboard, reloadWorldState } from '$lib/worldstate.svelte';
	import { config, loadSettings } from '$lib/settings.svelte';
	import { hasActiveNotificationRules } from '$lib/notifications.svelte';
	import Button from '$lib/components/Button.svelte';
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import { createFocusedRefresh } from '$lib/focused-refresh';
	import CycleWidgets from './widgets/CycleWidgets.svelte';
	import DashboardHeader from './widgets/DashboardHeader.svelte';
	import DashboardViewSettings from './widgets/DashboardViewSettings.svelte';
	import { dashboardViews, type DashboardView } from './dashboard-views';
	import NotificationRuleSettings from './widgets/NotificationRuleSettings.svelte';

	const REFRESH_INTERVAL_MS = 5 * 60_000;
	const MANUAL_RELOAD_COOLDOWN_MS = 3_000;
	let localNow = $state(Date.now());
	let isReloadCoolingDown = $state(false);
	let showAllViews = $state(false);
	let activeView = $state<DashboardView>('fissures');
	let favoriteViews = $derived(
		[...dashboardViews]
			.filter(
				(view) =>
					Array.isArray(config.dashboard_view_favorites) &&
					config.dashboard_view_favorites.includes(view.value),
			)
			.sort(
				(a, b) =>
					config.dashboard_view_favorites.indexOf(a.value) -
					config.dashboard_view_favorites.indexOf(b.value),
			),
	);
	let expandedViews = $derived([
		...favoriteViews,
		...dashboardViews.filter(
			(view) => !favoriteViews.some((favorite) => favorite.value === view.value),
		),
	]);
	let collapsedViews = $derived(
		favoriteViews.length > 0 ? favoriteViews : dashboardViews.slice(0, 5),
	);
	let visibleViews = $derived(showAllViews ? expandedViews : collapsedViews);
	let hasHiddenViews = $derived(collapsedViews.length < dashboardViews.length);
	let ActiveView = $derived(dashboardViews.find((view) => view.value === activeView)?.component);
	let reloadDashboard: () => void | Promise<void> = $state(reloadWorldState);
	let worldNow = $derived(
		dashboard.world && dashboard.fetchedAt !== null
			? dashboard.world.timestamp.getTime() + (localNow - dashboard.fetchedAt)
			: localNow,
	);

	onMount(() => {
		void loadSettings().catch((error) =>
			console.error('Could not load dashboard favorites:', error),
		);
		const focusedRefresh = createFocusedRefresh(() => {
			if (!hasActiveNotificationRules()) return reloadWorldState();
		}, REFRESH_INTERVAL_MS);
		let clock: ReturnType<typeof setInterval> | undefined;
		let cooldownTimer: ReturnType<typeof setTimeout> | undefined;
		const updateClock = () => {
			clearInterval(clock);
			localNow = Date.now();
			if (!document.hidden) clock = setInterval(() => (localNow = Date.now()), 1000);
		};

		reloadDashboard = () => {
			if (dashboard.loading || isReloadCoolingDown) return;
			isReloadCoolingDown = true;
			cooldownTimer = setTimeout(() => (isReloadCoolingDown = false), MANUAL_RELOAD_COOLDOWN_MS);
			void reloadWorldState();
		};
		document.addEventListener('visibilitychange', updateClock);
		updateClock();

		return () => {
			clearInterval(clock);
			clearTimeout(cooldownTimer);
			document.removeEventListener('visibilitychange', updateClock);
			focusedRefresh.destroy();
		};
	});

	$effect(() => {
		if (!showAllViews && !visibleViews.some((view) => view.value === activeView)) {
			activeView = visibleViews[0]?.value ?? 'fissures';
		}
	});
</script>

<div class="flex flex-col items-center gap-4 mx-auto p-8 w-full">
	<div class="flex flex-col gap-6 w-full max-w-3xl">
		<DashboardHeader
			loading={dashboard.loading}
			reloadCoolingDown={isReloadCoolingDown}
			error={dashboard.error}
			worldTimestamp={dashboard.world?.timestamp}
			fetchedAt={dashboard.fetchedAt}
			now={localNow}
			onReload={reloadDashboard}
		/>
		<div class="bg-surface w-full h-px"></div>
		{#if dashboard.world}
			<div class="flex flex-col gap-12">
				<CycleWidgets world={dashboard.world} now={worldNow} />
				<!-- <div class="bg-surface w-full h-px"></div> -->
				<div class="flex flex-col gap-4">
					<div class="flex flex-col gap-2">
						<div class="flex justify-between items-center gap-2">
							<div class="font-medium text-sm whitespace-nowrap">Live World State Views</div>
							<div class="bg-surface w-full h-px shrink grow-0"></div>
							<div class="flex items-center gap-2">
								{#if hasHiddenViews}
									<Button
										onclick={() => (showAllViews = !showAllViews)}
										class="text-sm whitespace-nowrap"
									>
										{showAllViews ? 'Show less views' : 'Show all views'}
									</Button>
								{/if}
								<NotificationRuleSettings world={dashboard.world} />
								<DashboardViewSettings />
							</div>
						</div>
						<RadioGroup
							label="Dashboard view"
							options={visibleViews}
							variant="tabs"
							separatorBefore={showAllViews &&
							favoriteViews.length > 0 &&
							favoriteViews.length < dashboardViews.length
								? favoriteViews.length
								: undefined}
							class="flex-1 min-w-0"
							bind:value={activeView}
						/>
					</div>
					<div class="bg-surface w-full h-px"></div>

					{#if ActiveView}
						<ActiveView world={dashboard.world} now={worldNow} />
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>
