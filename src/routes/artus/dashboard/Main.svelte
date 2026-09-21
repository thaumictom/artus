<script lang="ts">
	import { onMount } from 'svelte';
	import { RadioGroup } from 'bits-ui';
	import { dashboard, reloadWorldState } from '$lib/worldstate.svelte';
	import { config, loadSettings } from '$lib/settings.svelte';
	import Button from '$lib/components/Button.svelte';
	import { createFocusedRefresh } from '$lib/focused-refresh';
	import CycleWidgets from './widgets/CycleWidgets.svelte';
	import DashboardHeader from './widgets/DashboardHeader.svelte';
	import DashboardViewSettings from './widgets/DashboardViewSettings.svelte';
	import { dashboardViews, type DashboardView } from './dashboard-views';

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
		const focusedRefresh = createFocusedRefresh(reloadWorldState, REFRESH_INTERVAL_MS);
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
			void focusedRefresh.refresh();
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
			<CycleWidgets world={dashboard.world} now={worldNow} />
			<div class="bg-surface w-full h-px"></div>
			<div class="flex flex-col gap-4">
				<div class="flex items-start gap-2">
					<RadioGroup.Root
						aria-label="Dashboard view"
						class="flex flex-wrap flex-1 gap-2 min-w-0"
						bind:value={activeView}
					>
						{#each visibleViews as view, index (view.value)}
							{#if showAllViews && favoriteViews.length > 0 && favoriteViews.length < dashboardViews.length && index === favoriteViews.length}
								<div aria-hidden="true" class="self-stretch bg-surface w-px min-h-7"></div>
							{/if}
							<RadioGroup.Item
								value={view.value}
								class="data-[state=checked]:bg-accent/10 hover:bg-surface px-2 py-1.5 border data-[state=checked]:border-accent text-muted-foreground data-[state=checked]:text-accent hover:text-foreground text-sm cursor-pointer"
							>
								{view.label}
							</RadioGroup.Item>
						{/each}
					</RadioGroup.Root>
					<div class="flex gap-2 ml-auto shrink-0">
						{#if hasHiddenViews}
							<Button
								onclick={() => (showAllViews = !showAllViews)}
								class="text-sm whitespace-nowrap"
							>
								{showAllViews ? 'Show less' : 'Show more'}
							</Button>
						{/if}
						<DashboardViewSettings />
					</div>
				</div>
				<div class="bg-surface w-full h-px"></div>

				{#if ActiveView}
					<ActiveView world={dashboard.world} now={worldNow} />
				{/if}
			</div>
		{/if}
	</div>
</div>
