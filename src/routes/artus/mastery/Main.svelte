<script lang="ts">
	import { LazyStore } from '@tauri-apps/plugin-store';
	import { onMount } from 'svelte';
	import { inventoryMarketSlug, inventoryNameKey, waitForInventorySave, type InventoryItem } from '$lib/inventory';
	import { isOwnedMasteryComponent, mastery, type MasteryItem } from '$lib/mastery.svelte';
	import Select from '$lib/components/Select.svelte';
	import Button from '$lib/components/Button.svelte';
	import MasteryTable from './MasteryTable.svelte';
	import BuyDialog from './BuyDialog.svelte';
	import TrackedMastery from './TrackedMastery.svelte';
	let { onOpenMarket = () => {} }: { onOpenMarket?: (slug: string) => void } = $props();

	let search = $state('');
	let buyItem = $state<{ slug: string; name: string } | null>(null);
	const searchQuery = $derived(search.trim().toLowerCase());
	let category = $state('All');
	let tag = $state('All');
	let progress = $state('All');
	type SortColumn = 'name' | 'median' | 'ducats';
	let sortColumn = $state<SortColumn>('name');
	let sortDirection = $state<'asc' | 'desc'>('asc');
	let visibleCount = $state(150);
	let expanded = $state<string[]>([]);
	let inventoryItems = $state<InventoryItem[]>([]);
	onMount(() => {
		let disposed = false;
		void waitForInventorySave()
			.then(() => new LazyStore('inventory.json').get<InventoryItem[]>('items'))
			.then((items) => {
				if (!disposed) inventoryItems = items ?? [];
			})
			.catch((error) => console.error('Could not load inventory ownership:', error));
		return () => { disposed = true; };
	});
	const ownedComponents = $derived.by(() => {
		const bySlug = new Map<string, number>();
		const byName = new Map<string, number>();
		for (const item of inventoryItems) {
			if (item.quantity <= 0 || item.isCustom) continue;
			const slug = inventoryMarketSlug(item);
			if (slug) bySlug.set(slug, (bySlug.get(slug) ?? 0) + item.quantity);
			const name = inventoryNameKey(item.name);
			byName.set(name, (byName.get(name) ?? 0) + item.quantity);
		}
		return { bySlug, byName };
	});
	function ownedComponentCount(component: { marketSlug?: string | null; name: string }, parentName: string) {
		const componentName = inventoryNameKey(component.name);
		const parent = inventoryNameKey(parentName);
		const fullName = componentName.startsWith(`${parent} `)
			? componentName
			: inventoryNameKey(`${parentName} ${component.name}`);
		return (component.marketSlug ? ownedComponents.bySlug.get(component.marketSlug) : undefined)
			?? ownedComponents.byName.get(fullName)
			?? 0;
	}
	const checked = $derived(new Set(mastery.checked));
	const automatic = $derived(new Set(mastery.automatic));
	function completedComponentCount(item: MasteryItem) {
		return item.components.filter((part) =>
			checked.has(part.key) || isOwnedMasteryComponent(part, ownedComponentCount(part, item.name))
		).length;
	}
	const categories = $derived([
		'All',
		...new Set(mastery.items.map((item) => item.category ?? 'Other')),
	]);
	const tags = $derived([
		'All',
		...new Set([
			'Prime',
			'Non-prime',
			'Tradeable',
			'Has components',
			...mastery.items.flatMap((item) => item.tags ?? []),
		]),
	]);
	const categoryOptions = $derived(categories.map((value) => ({ value, label: value })));
	const tagOptions = $derived(tags.map((value) => ({ value, label: value })));
	const progressOptions = [
		{ value: 'All', label: 'All' },
		{ value: 'Checked', label: 'Checked' },
		{ value: 'Unchecked', label: 'Unchecked' },
		{ value: 'In progress', label: 'In progress' },
	];
	const filtered = $derived(
		mastery.items.filter((item) => {
			if (category !== 'All' && (item.category ?? 'Other') !== category) return false;
			const isPrime = item.name.includes('Prime');
			if (tag === 'Prime' && !isPrime) return false;
			if (tag === 'Non-prime' && isPrime) return false;
			if (tag === 'Tradeable' && !item.marketSlug && !item.components.some((part) => part.tradable))
				return false;
			if (tag === 'Has components' && item.components.length === 0) return false;
			if (
				!['All', 'Prime', 'Non-prime', 'Tradeable', 'Has components'].includes(tag) &&
				!(item.tags ?? []).includes(tag)
			)
				return false;
			if (progress === 'Checked' && !checked.has(item.key)) return false;
			if (progress === 'Unchecked' && checked.has(item.key)) return false;
			if (progress === 'In progress' && (checked.has(item.key) || completedComponentCount(item) === 0)) return false;
			return !searchQuery || item.name.toLowerCase().includes(searchQuery);
		}),
	);
	const sorted = $derived.by(() => {
		const items = [...filtered];
		if (sortColumn === 'name')
			return items.sort(
				(a, b) => (sortDirection === 'asc' ? 1 : -1) * a.name.localeCompare(b.name),
			);
		return items.sort((a, b) => {
			const first = sortColumn === 'median' ? priceFor(a)?.median : ducatsFor(a);
			const second = sortColumn === 'median' ? priceFor(b)?.median : ducatsFor(b);
			// Items without a value stay at the end in either direction.
			if (first == null) return second == null ? a.name.localeCompare(b.name) : 1;
			if (second == null) return -1;
			return (
				(sortDirection === 'asc' ? first - second : second - first) || a.name.localeCompare(b.name)
			);
		});
	});
	const visible = $derived(sorted.slice(0, visibleCount));
	function toggle(key: string) {
		expanded = expanded.includes(key)
			? expanded.filter((entry) => entry !== key)
			: [...expanded, key];
	}
	function setSort(column: SortColumn) {
		if (sortColumn === column) sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		else {
			sortColumn = column;
			sortDirection = column === 'name' ? 'asc' : 'desc';
		}
	}
	function priceFor(item: { marketSlug?: string | null }) {
		return item.marketSlug ? mastery.prices[item.marketSlug] : undefined;
	}
	function ducatsFor(item: { marketSlug?: string | null; ducats?: number | null }) {
		return item.marketSlug ? (mastery.ducats[item.marketSlug] ?? item.ducats) : item.ducats;
	}
	function updateSearch(event: Event & { currentTarget: HTMLInputElement }) {
		search = event.currentTarget.value;
		visibleCount = 150;
	}
	function resetFilters() {
		search = '';
		category = 'All';
		tag = 'All';
		progress = 'All';
		sortColumn = 'name';
		sortDirection = 'asc';
		visibleCount = 150;
	}
</script>

<div class="flex flex-col items-center gap-4 mx-auto p-8 w-full">
	<div class="flex flex-col gap-6 w-full max-w-5xl">
		<TrackedMastery />
		{#if mastery.loading}<p class="py-10 text-muted-foreground text-center">
				Loading mastery items…
			</p>
		{:else if mastery.error}<p class="p-4 border border-destructive rounded text-destructive">
				{mastery.error}
			</p>
		{:else}
			<div class="flex flex-wrap items-end gap-3">
				<div class="flex-1 min-w-56">
					<label
						for="mastery-search"
						class="block mb-1.5 font-semibold text-muted-foreground text-xs"
					>
						Search items
					</label>
					<input
						id="mastery-search"
						type="search"
						value={search}
						oninput={updateSearch}
						placeholder="Search for an item..."
						class="bg-background p-2 border focus-visible:border-accent outline-none w-full text-foreground placeholder:text-muted-foreground"
					/>
				</div>
				<div class="flex-1 min-w-36">
					<label
						for="mastery-category"
						class="block mb-1.5 font-semibold text-muted-foreground text-xs"
					>
						Category
					</label>
					<Select
						type="single"
						items={categoryOptions}
						bind:value={category}
						triggerProps={{ id: 'mastery-category', class: 'max-w-none h-10' }}
					/>
				</div>
				<div class="flex-1 min-w-36">
					<label for="mastery-tag" class="block mb-1.5 font-semibold text-muted-foreground text-xs">
						Tag
					</label>
					<Select
						type="single"
						items={tagOptions}
						bind:value={tag}
						triggerProps={{ id: 'mastery-tag', class: 'max-w-none h-10' }}
					/>
				</div>
				<div class="flex-1 min-w-36">
					<label
						for="mastery-progress"
						class="block mb-1.5 font-semibold text-muted-foreground text-xs"
					>
						Progress
					</label>
					<Select
						type="single"
						items={progressOptions}
						bind:value={progress}
						triggerProps={{ id: 'mastery-progress', class: 'max-w-none h-10' }}
					/>
				</div>
				<Button onclick={resetFilters} class="h-10">Reset filters</Button>
			</div>
			<MasteryTable
				items={visible}
				{checked}
				{automatic}
				{expanded}
				{sortColumn}
				{sortDirection}
				onSort={setSort}
				onToggle={toggle}
				{onOpenMarket}
				onBuy={(slug, name) => (buyItem = { slug, name })}
				{ownedComponentCount}
				{completedComponentCount}
			/>
			<div class="flex justify-between items-center text-muted-foreground text-sm">
				<span>Showing {visible.length} of {filtered.length} items</span>
				{#if visible.length < filtered.length}<button
						class="hover:bg-muted px-3 py-1.5 border border-border rounded text-foreground"
						onclick={() => (visibleCount += 80)}
					>
						Show more
					</button>{/if}
			</div>
		{/if}
	</div>
</div>
<BuyDialog bind:item={buyItem} />
