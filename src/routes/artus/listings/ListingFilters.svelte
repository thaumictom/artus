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

<div
	class="items-end gap-4 grid md:grid-cols-2 xl:grid-cols-[minmax(14rem,1fr)_11rem_11rem] bg-card/50"
>
	<label for="listings-search" class="block min-w-0">
		<span class="block mb-1.5 font-semibold text-muted-foreground text-sm">Search listings</span>
		<span class="block relative">
			<Icon
				icon="lucide:search"
				class="top-1/2 left-3 absolute size-4 text-muted-foreground -translate-y-1/2 pointer-events-none"
			/>
			<input
				id="listings-search"
				data-item-search
				type="search"
				bind:value={search}
				placeholder="Search by item name or keyword..."
				class="bg-background/70 py-2 pr-3 pl-10 border border-border-secondary focus-visible:border-accent outline-none w-full h-11 text-foreground placeholder:text-muted-foreground"
			/>
		</span>
	</label>
	<label for="listings-type-filter" class="block min-w-0">
		<span class="block mb-1.5 font-semibold text-muted-foreground text-sm">Listing type</span>
		<Select
			type="single"
			bind:value={typeFilter}
			items={typeItems}
			placeholder="Listing type"
			triggerProps={{
				id: 'listings-type-filter',
				class: 'h-11 max-w-none border-border-secondary bg-background/70',
			}}
		/>
	</label>
	<label for="listings-status-filter" class="block min-w-0">
		<span class="block mb-1.5 font-semibold text-muted-foreground text-sm">Status</span>
		<Select
			type="single"
			bind:value={statusFilter}
			items={statusItems}
			placeholder="Status"
			triggerProps={{
				id: 'listings-status-filter',
				class: 'h-11 max-w-none border-border-secondary bg-background/70',
			}}
		/>
	</label>
</div>
