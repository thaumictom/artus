<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { dashboard, reloadWorldState } from '$lib/worldstate.svelte';
	import { appNavigation, navigateTo } from '$lib/app-navigation.svelte';
	import { hasActiveNotificationRules } from '$lib/notifications.svelte';
	import { createFocusedRefresh } from '$lib/focused-refresh';
	import DashboardHeader from './widgets/DashboardHeader.svelte';
	import DashboardNavigation from './widgets/DashboardNavigation.svelte';
	import { dashboardViews, type DashboardView } from './dashboard-views';
	import NotificationRuleSettings from './widgets/NotificationRuleSettings.svelte';
	import MainContent from '../MainContent.svelte';

	const REFRESH_INTERVAL_MS = 5 * 60_000;
	const MANUAL_RELOAD_COOLDOWN_MS = 3_000;
	let localNow = $state(Date.now());
	let isReloadCoolingDown = $state(false);
	let activeView = $derived(appNavigation.current.dashboardView);
	let liveViewsElement = $state<HTMLDivElement>();
	let selectedView = $derived(dashboardViews.find((view) => view.value === activeView));
	let ActiveView = $derived(selectedView?.component);
	// Wiki schedules and request limits follow UTC wall time, independent of world-state age.
	let isWikiView = $derived(
		activeView === 'TenetWeapons' || activeView === 'CodaWeapons' || activeView === 'Acrithis',
	);
	let reloadDashboard: () => void | Promise<void> = $state(reloadWorldState);
	let worldNow = $derived(
		dashboard.world && dashboard.fetchedAt !== null
			? dashboard.world.timestamp.getTime() + (localNow - dashboard.fetchedAt)
			: localNow,
	);

	async function openDashboardView(target: DashboardView) {
		navigateTo('dashboard', '', target);
		await tick();
		liveViewsElement?.scrollIntoView({ behavior: 'smooth', block: 'start' });
	}

	onMount(() => {
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
</script>

<div class="flex flex-1 w-full min-w-0 h-full min-h-0 overflow-hidden">
	<DashboardNavigation {activeView} world={dashboard.world} now={worldNow} onSelect={openDashboardView}>
		{#snippet footer()}
			<DashboardHeader
				loading={dashboard.loading}
				reloadCoolingDown={isReloadCoolingDown}
				error={dashboard.error}
				worldTimestamp={dashboard.world?.timestamp}
				fetchedAt={dashboard.fetchedAt}
				now={localNow}
				onReload={reloadDashboard}
			>
				{#if dashboard.world}
					<NotificationRuleSettings
						world={dashboard.world}
						triggerClass="flex items-center justify-center shrink-0"
					/>
				{/if}
			</DashboardHeader>
		{/snippet}
	</DashboardNavigation>
	<MainContent>
		<div class="p-6 lg:p-8 min-h-full">
			<div class="flex flex-col gap-6 mx-auto w-full max-w-3xl page-width">
				{#if dashboard.world}
					<div class="flex flex-col gap-4 scroll-mt-6" bind:this={liveViewsElement}>
						{#if ActiveView}
							<ActiveView
								world={dashboard.world}
								now={isWikiView ? localNow : worldNow}
								{localNow}
								onSelect={openDashboardView}
							/>
						{/if}
					</div>
				{/if}
			</div>
		</div>
	</MainContent>
</div>
