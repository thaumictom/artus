<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { dashboard, reloadWorldState } from '$lib/worldstate.svelte';
	import { hasActiveNotificationRules } from '$lib/notifications.svelte';
	import { createFocusedRefresh } from '$lib/focused-refresh';
	import CycleWidgets from './widgets/CycleWidgets.svelte';
	import WeaponResets from './widgets/WeaponResets.svelte';
	import DashboardHeader from './widgets/DashboardHeader.svelte';
	import DashboardNavigation from './widgets/DashboardNavigation.svelte';
	import { dashboardViews, type DashboardView } from './dashboard-views';
	import NotificationRuleSettings from './widgets/NotificationRuleSettings.svelte';

	const REFRESH_INTERVAL_MS = 5 * 60_000;
	const MANUAL_RELOAD_COOLDOWN_MS = 3_000;
	let localNow = $state(Date.now());
	let isReloadCoolingDown = $state(false);
	let activeView = $state<DashboardView>('fissures');
	let liveViewsElement = $state<HTMLDivElement>();
	let selectedView = $derived(dashboardViews.find((view) => view.value === activeView));
	let ActiveView = $derived(selectedView?.component);
	// Wiki schedules and request limits follow UTC wall time, independent of world-state age.
	let isWikiView = $derived(activeView === 'TenetWeapons' || activeView === 'CodaWeapons' || activeView === 'Acrithis');
	let reloadDashboard: () => void | Promise<void> = $state(reloadWorldState);
	let worldNow = $derived(
		dashboard.world && dashboard.fetchedAt !== null
			? dashboard.world.timestamp.getTime() + (localNow - dashboard.fetchedAt)
			: localNow,
	);

	async function openDashboardView(target: DashboardView) {
		activeView = target;
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

<div class="flex items-start w-full min-h-full">
	<DashboardNavigation {activeView} onSelect={openDashboardView} />
	<div class="flex-1 p-6 lg:p-8 min-w-0">
		<div class="page-width flex flex-col gap-6 mx-auto w-full max-w-3xl">
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
			<div class="flex flex-col gap-2">
				{#if dashboard.world}
					<CycleWidgets
						world={dashboard.world}
						now={worldNow}
						onOpenBaro={() => openDashboardView('BaroInventory')}
					/>
				{/if}
				<!-- Fixed UTC rotations use wall time without requesting wiki or world-state data. -->
				<WeaponResets
					now={localNow}
					onOpen={(source) => openDashboardView(source === 'tenet' ? 'TenetWeapons' : 'CodaWeapons')}
				/>
			</div>
			{#if dashboard.world}
				<div class="flex flex-col gap-4 scroll-mt-6" bind:this={liveViewsElement}>
					<div class="flex justify-between items-center gap-3">
						<h1 class="font-medium text-sm">{selectedView?.label}</h1>
						<div class="bg-surface h-px grow"></div>
						<NotificationRuleSettings world={dashboard.world} />
					</div>
					<div class="bg-surface w-full h-px"></div>

					{#if ActiveView}
						<ActiveView world={dashboard.world} now={isWikiView ? localNow : worldNow} />
					{/if}
				</div>
			{/if}
		</div>
	</div>
</div>
