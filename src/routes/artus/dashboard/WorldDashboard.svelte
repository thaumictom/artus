<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import { dashboard, reloadWorldState } from '$lib/worldstate.svelte';
	import { buildWorldSections, worldCycles, duration, fetchedAgo } from '$lib/worldstate';
	import WorldSection from './WorldSection.svelte';

	let now = $state(Date.now());
	let sections = $derived(dashboard.world ? buildWorldSections(dashboard.world, dashboard.catalog) : []);
	let cycles = $derived(dashboard.world ? worldCycles(dashboard.world) : []);
	let query = $state('');
	let visibleSections = $derived.by(() => {
		const search = query.trim().toLowerCase();
		if (!search) return sections;
		return sections.map((section) => ({ ...section, collapsed: false,
			rows: section.title.toLowerCase().includes(search) ? section.rows : section.rows.filter((row) =>
				[row.title, row.detail, row.badge, ...(row.rewards ?? [])].filter(Boolean).join(' ').toLowerCase().includes(search)),
		})).filter((section) => section.rows.length);
	});
	onMount(() => {
		// Update display clocks only. World-state requests happen at startup and on Reload.
		const timer = setInterval(() => now = Date.now(), 1000);
		return () => clearInterval(timer);
	});
</script>

<div class="flex flex-col gap-6 mx-auto p-6 lg:p-8 w-full max-w-7xl">
	<header class="flex flex-wrap justify-between items-center gap-4">
		<div>
			<h1 class="text-xl font-medium">World State</h1>
			<p class="mt-1 text-muted-foreground text-sm">Activities, rotations & rewards across the Origin System</p>
		</div>
		<div class="flex items-center gap-3">
			{#if dashboard.fetchedAt !== null}
				<span class="text-muted-foreground text-xs tabular-nums" title={new Date(dashboard.fetchedAt).toLocaleString()}>
					fetched {fetchedAgo(dashboard.fetchedAt, now)}
				</span>
			{/if}
			<Button class="flex items-center gap-1.5" onclick={reloadWorldState} disabled={dashboard.loading}>
				<Icon icon="material-symbols:refresh" class={dashboard.loading ? 'size-4 animate-spin' : 'size-4'} />
				{dashboard.loading ? 'Loading…' : 'Reload'}
			</Button>
		</div>
	</header>
	{#if dashboard.error}
		<p role="alert" class="p-3 border border-danger/40 text-danger text-sm">
			{dashboard.error} {dashboard.world ? 'The last successful snapshot is still shown.' : ''}
		</p>
	{/if}
	{#if dashboard.loading && !dashboard.world}
		<p role="status" class="text-muted-foreground text-sm">Fetching the latest world state…</p>
	{:else if dashboard.world}
		<div class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-6 gap-3" aria-label="Environment cycles">
			{#each cycles as cycle}
				<div class="bg-background p-3 border border-surface">
					<h2 class="text-muted-foreground text-xs">{cycle.title}</h2>
					<p class="mt-2 font-medium">{cycle.state}</p>
					<p class="mt-1 text-muted-foreground text-xs tabular-nums" title={cycle.expiry ? new Date(cycle.expiry).toLocaleString() : ''}>
						{cycle.expiry ? cycle.expiry > now ? `Changes in ${duration(cycle.expiry - now)}` : 'Changed · reload' : 'No cycle data'}
					</p>
				</div>
			{/each}
		</div>
		<div class="flex flex-wrap justify-between items-center gap-3">
			<label class="flex items-center gap-2 bg-background px-3 py-2 border focus-within:border-accent w-full sm:max-w-sm">
				<Icon icon="material-symbols:search" class="size-4 text-muted-foreground" />
				<input type="search" bind:value={query} aria-label="Filter dashboard activities and rewards" placeholder="Find an activity, item or reward…" class="bg-transparent outline-none w-full text-sm placeholder:text-muted-foreground" />
			</label>
			<p class="text-muted-foreground text-xs">Manual refresh · snapshot {new Date(dashboard.world.Time * 1000).toLocaleString()}</p>
		</div>
		{#if dashboard.catalogError}<p class="text-muted-foreground text-xs">Some item names are unavailable because the local item catalog could not be read.</p>{/if}
		<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 items-start gap-4">
			{#each visibleSections as section (section.id)}<WorldSection {section} {now} />{:else}
				<p class="text-muted-foreground text-sm">No matching activities or rewards.</p>
			{/each}
		</div>
		<p class="text-muted-foreground text-xs leading-relaxed">
			Arbitration and Steel Path incursion schedules are not included in this feed. Timers describe the fetched snapshot; reload after rotations change.
		</p>
	{/if}
</div>
