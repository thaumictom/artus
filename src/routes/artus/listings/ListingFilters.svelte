<script lang="ts">
	import Icon from '@iconify/svelte';
	import Select from '$lib/components/Select.svelte';

	let {
		search = $bindable(''),
		typeFilter = $bindable<'all' | 'buy' | 'sell'>('all'),
		statusFilter = $bindable<'all' | 'visible' | 'hidden'>('all'),
	}: {
		search: string;
		typeFilter: 'all' | 'buy' | 'sell';
		statusFilter: 'all' | 'visible' | 'hidden';
	} = $props();

	const typeItems = [
		{ value: 'all', label: 'Buy and sell' },
		{ value: 'buy', label: 'Buy' },
		{ value: 'sell', label: 'Sell' },
	];
	const statusItems = [
		{ value: 'all', label: 'All listings' },
		{ value: 'visible', label: 'Visible' },
		{ value: 'hidden', label: 'Hidden' },
	];
</script>

<div class="grid items-end gap-4 border border-border-secondary bg-card/50 p-4 md:grid-cols-2 xl:grid-cols-[minmax(14rem,1fr)_11rem_11rem]">
	<label for="listings-search" class="block min-w-0">
		<span class="mb-1.5 block font-semibold text-muted-foreground text-xs">Search listings</span>
		<span class="relative block">
			<Icon icon="lucide:search" class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
			<input id="listings-search" type="search" bind:value={search} placeholder="Search by item name or keyword..." class="h-11 w-full border border-border-secondary bg-background/70 py-2 pr-3 pl-10 text-foreground outline-none placeholder:text-muted-foreground focus-visible:border-accent" />
		</span>
	</label>
	<label for="listings-type-filter" class="block min-w-0">
		<span class="mb-1.5 block font-semibold text-muted-foreground text-xs">Listing type</span>
		<Select type="single" bind:value={typeFilter} items={typeItems} placeholder="Listing type" triggerProps={{ id: 'listings-type-filter', class: 'h-11 max-w-none border-border-secondary bg-background/70' }} />
	</label>
	<label for="listings-status-filter" class="block min-w-0">
		<span class="mb-1.5 block font-semibold text-muted-foreground text-xs">Status</span>
		<Select type="single" bind:value={statusFilter} items={statusItems} placeholder="Status" triggerProps={{ id: 'listings-status-filter', class: 'h-11 max-w-none border-border-secondary bg-background/70' }} />
	</label>
</div>
