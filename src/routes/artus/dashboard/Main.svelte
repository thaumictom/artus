<script lang="ts">
	import { onMount } from 'svelte';
	import { dashboard, reloadWorldState } from '$lib/worldstate.svelte';
	import { createFocusedRefresh } from '$lib/focused-refresh';
	import CycleWidgets from './widgets/CycleWidgets.svelte';
	import DashboardHeader from './widgets/DashboardHeader.svelte';
	import News from './widgets/News.svelte';

	const REFRESH_INTERVAL_MS = 5 * 60_000;
	let now = $state(Date.now());
	let reloadDashboard = $state(reloadWorldState);
	let requestDashboardReload = $state<() => boolean>(() => false);

	onMount(() => {
		const focusedRefresh = createFocusedRefresh(reloadWorldState, REFRESH_INTERVAL_MS);
		let clock: ReturnType<typeof setInterval> | undefined;
		const updateClock = () => {
			clearInterval(clock);
			now = Date.now();
			if (!document.hidden) clock = setInterval(() => (now = Date.now()), 1000);
		};

		reloadDashboard = focusedRefresh.refresh;
		requestDashboardReload = focusedRefresh.requestRefresh;
		document.addEventListener('visibilitychange', updateClock);
		updateClock();

		return () => {
			clearInterval(clock);
			document.removeEventListener('visibilitychange', updateClock);
			focusedRefresh.destroy();
		};
	});
</script>

<div class="flex flex-col gap-6 p-6">
	<DashboardHeader
		loading={dashboard.loading}
		error={dashboard.error}
		worldTimestamp={dashboard.world?.timestamp}
		fetchedAt={dashboard.fetchedAt}
		{now}
		onReload={reloadDashboard}
	/>

	{#if dashboard.world}
		<CycleWidgets world={dashboard.world} {now} onRefreshRequest={requestDashboardReload} />
		<News articles={dashboard.world.news} />
	{/if}
</div>
