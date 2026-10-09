<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { dashboard, reloadWorldState } from '$lib/worldstate.svelte';
	import { appNavigation, navigateTo } from '$lib/app-navigation.svelte';
	import { hasActiveNotificationRules } from '$lib/notifications.svelte';
	import { createFocusedRefresh } from '$lib/focused-refresh';
	import { config, loadSettings } from '$lib/settings.svelte';
	import DashboardHeader from './widgets/DashboardFooter.svelte';
	import DashboardNavigation from './widgets/DashboardNavigation.svelte';
	import { dashboardViews, isDashboardViewVisible, type DashboardView } from './dashboard-views';
	import NotificationRuleSettings from './widgets/NotificationRuleSettings.svelte';
	import MainContent from '../MainContent.svelte';
	import ResizableNavigationLayout from '../ResizableNavigationLayout.svelte';

	const REFRESH_INTERVAL_MS = 5 * 60_000;
	const MANUAL_RELOAD_COOLDOWN_MS = 3_000;
	let localNow = $state(Date.now());
	let isReloadCoolingDown = $state(false);
	let settingsReady = $state(false);
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
		if (!isDashboardViewVisible(target, config.show_unused_dashboard_views)) return;
		navigateTo('dashboard', '', target);
		await tick();
		liveViewsElement?.scrollIntoView({ behavior: 'smooth', block: 'start' });
	}

	$effect(() => {
		if (settingsReady && !isDashboardViewVisible(activeView, config.show_unused_dashboard_views)) {
			navigateTo('dashboard', '', 'Home');
		}
	});

	onMount(() => {
		void loadSettings()
			.then(() => {
				settingsReady = true;
			})
			.catch((error) => {
				console.error('Could not load dashboard settings:', error);
				settingsReady = true;
			});
		const focusedRefresh = createFocusedRefresh(
			() => {
				const isStale =
					dashboard.fetchedAt === null || Date.now() - dashboard.fetchedAt >= REFRESH_INTERVAL_MS;
				if (!hasActiveNotificationRules() || isStale) return reloadWorldState();
			},
			REFRESH_INTERVAL_MS,
			{
				immediate: dashboard.fetchedAt === null,
				lastRefreshedAt: dashboard.fetchedAt ?? undefined,
			},
		);
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

<ResizableNavigationLayout resizeLabel="Resize dashboard navigation">
	{#snippet navigation()}
		<DashboardNavigation
			{activeView}
			world={dashboard.world}
			now={worldNow}
			onSelect={openDashboardView}
		>
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
	{/snippet}
	<MainContent>
		<div class="p-6 lg:p-8 min-h-full">
			<div class="flex flex-col gap-6 mx-auto w-full {activeView === 'Home' ? '' : 'max-w-3xl'} page-width">
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
</ResizableNavigationLayout>
