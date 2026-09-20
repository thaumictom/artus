<script lang="ts">
	import { onMount } from 'svelte';
	import { dashboard, reloadWorldState } from '$lib/worldstate.svelte';
	import { createFocusedRefresh } from '$lib/focused-refresh';
	import CycleWidgets from './widgets/CycleWidgets.svelte';
	import DashboardHeader from './widgets/DashboardHeader.svelte';
	import BaroKiTeer from './widgets/BaroKiTeer.svelte';
	import FissureMissions from './widgets/FissureMissions.svelte';
	import News from './widgets/News.svelte';

	const REFRESH_INTERVAL_MS = 5 * 60_000;
	let localNow = $state(Date.now());
	let reloadDashboard = $state(reloadWorldState);
	let worldNow = $derived(
		dashboard.world && dashboard.fetchedAt !== null
			? dashboard.world.timestamp.getTime() + (localNow - dashboard.fetchedAt)
			: localNow,
	);

	onMount(() => {
		const focusedRefresh = createFocusedRefresh(reloadWorldState, REFRESH_INTERVAL_MS);
		let clock: ReturnType<typeof setInterval> | undefined;
		const updateClock = () => {
			clearInterval(clock);
			localNow = Date.now();
			if (!document.hidden) clock = setInterval(() => (localNow = Date.now()), 1000);
		};

		reloadDashboard = focusedRefresh.refresh;
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
		now={localNow}
		onReload={reloadDashboard}
	/>

	{#if dashboard.world}
		<CycleWidgets world={dashboard.world} now={worldNow} />
		<div class="gap-4 grid lg:grid-cols-[minmax(14rem,1fr)_minmax(0,3fr)] items-start">
			<BaroKiTeer trader={dashboard.world.voidTrader} now={worldNow} />
			<FissureMissions fissures={dashboard.world.fissures} now={worldNow} />
		</div>
		<News articles={dashboard.world.news} />
	{/if}
</div>
