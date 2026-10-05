<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { Pane, PaneGroup, PaneResizer } from 'paneforge';
	import { dashboard, reloadWorldState } from '$lib/worldstate.svelte';
	import { appNavigation, navigateTo } from '$lib/app-navigation.svelte';
	import { hasActiveNotificationRules } from '$lib/notifications.svelte';
	import { createFocusedRefresh } from '$lib/focused-refresh';
	import { config, loadSettings, updateSetting } from '$lib/settings.svelte';
	import DashboardHeader from './widgets/DashboardFooter.svelte';
	import DashboardNavigation from './widgets/DashboardNavigation.svelte';
	import { dashboardViews, isDashboardViewVisible, type DashboardView } from './dashboard-views';
	import NotificationRuleSettings from './widgets/NotificationRuleSettings.svelte';
	import MainContent from '../MainContent.svelte';

	const REFRESH_INTERVAL_MS = 5 * 60_000;
	const MANUAL_RELOAD_COOLDOWN_MS = 3_000;
	let localNow = $state(Date.now());
	let isReloadCoolingDown = $state(false);
	let settingsReady = $state(false);
	const RESIZER_WIDTH = 6;
	let dashboardWidth = $state(0);
	let initialNavigationWidth = $state(240);
	let widthSaveTimer: ReturnType<typeof setTimeout> | undefined;
	// PaneForge divides the space left after the resizer using percentages.
	let paneWidth = $derived(Math.max(1, dashboardWidth - RESIZER_WIDTH));
	let navigationMinSize = $derived(Math.min(100, (200 / paneWidth) * 100));
	let navigationMaxSize = $derived(Math.min(100, (350 / paneWidth) * 100));
	let navigationDefaultSize = $derived(Math.min(100, (initialNavigationWidth / paneWidth) * 100));
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

	function saveNavigationWidth() {
		if (widthSaveTimer === undefined) return;
		clearTimeout(widthSaveTimer);
		widthSaveTimer = undefined;
		void updateSetting('dashboard_navigation_width').catch((error) => {
			console.error('Could not save dashboard navigation width:', error);
		});
	}

	function onNavigationResize(size: number) {
		// Store pixels so restoring the width does not depend on the window size.
		const width = Math.round(size / 100 * paneWidth);
		if (width === config.dashboard_navigation_width) return;
		config.dashboard_navigation_width = width;
		clearTimeout(widthSaveTimer);
		widthSaveTimer = setTimeout(saveNavigationWidth, 250);
	}

	async function openDashboardView(target: DashboardView) {
		if (!isDashboardViewVisible(target, config.show_unused_dashboard_views)) return;
		navigateTo('dashboard', '', target);
		await tick();
		liveViewsElement?.scrollIntoView({ behavior: 'smooth', block: 'start' });
	}

	$effect(() => {
		if (settingsReady && !isDashboardViewVisible(activeView, config.show_unused_dashboard_views)) {
			navigateTo('dashboard', '', 'fissures');
		}
	});

	onMount(() => {
		void loadSettings()
			.then(() => {
				initialNavigationWidth = config.dashboard_navigation_width;
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
			saveNavigationWidth();
			clearInterval(clock);
			clearTimeout(cooldownTimer);
			document.removeEventListener('visibilitychange', updateClock);
			focusedRefresh.destroy();
		};
	});
</script>

<div
	class="flex flex-1 w-full min-w-0 h-full min-h-0 overflow-hidden"
	bind:clientWidth={dashboardWidth}
>
	{#if dashboardWidth > 0 && settingsReady}
		<PaneGroup direction="horizontal" class="min-w-0 min-h-0">
			<Pane
				defaultSize={navigationDefaultSize}
				minSize={navigationMinSize}
				maxSize={navigationMaxSize}
				onResize={onNavigationResize}
			>
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
			</Pane>
			<PaneResizer
				aria-label="Resize dashboard navigation"
				onDraggingChange={(dragging) => { if (!dragging) saveNavigationWidth(); }}
				style={`width: ${RESIZER_WIDTH}px`}
				class="group flex justify-center items-center data-[active]:bg-surface hover:bg-surface rounded focus-visible:outline-2 focus-visible:outline-accent shrink-0"
			>
				<div
					class="bg-muted-foreground/30 group-data-[active]:bg-accent group-hover:bg-accent rounded w-0.5 h-8"
				></div>
			</PaneResizer>
			<Pane class="flex min-w-0 min-h-0">
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
			</Pane>
		</PaneGroup>
	{/if}
</div>
