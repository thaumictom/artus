<script lang="ts">
	import { LazyStore } from '@tauri-apps/plugin-store';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import Table, { type TableColumn } from '$lib/components/Table.svelte';
	import { inventoryNameKey, type InventoryItem } from '$lib/inventory';
	import { DictionarySchema } from '$lib/schemas';
	import InventoryRow from './InventoryRow.svelte';

	let { onOpenMarket = () => {} }: { onOpenMarket?: (slug: string) => void } = $props();

	type InventoryOcrWord = {
		slug?: string;
		is_custom?: boolean;
		text: string;
		market_median?: number;
		market_median_from_current_offers?: boolean;
		ducats?: number;
	};
	const store = new LazyStore('inventory.json');
	const columns: TableColumn[] = [
		{ key: 'name', label: 'Item', sortable: true, class: 'min-w-48' },
		{ key: 'quantity', label: 'Quantity', sortable: true, align: 'right', class: 'w-36' },
		{ key: 'median', label: 'Median', sortable: true, align: 'right', class: 'w-24' },
		{ key: 'ducats', label: 'Ducats', sortable: true, align: 'right', class: 'w-24' },
		{ key: 'totalPrice', label: 'Total price', align: 'right', class: 'w-28' },
		{ key: 'totalDucats', label: 'Total ducats', align: 'right', class: 'w-28' },
		{ key: 'links', label: 'Links', align: 'right', class: 'w-32' },
	];
	type SortColumn = 'name' | 'quantity' | 'median' | 'ducats';
	let data = $state<InventoryItem[]>([]);
	let loading = $state(true);
	let search = $state('');
	let sortColumn = $state<SortColumn>('name');
	let sortDirection = $state<'asc' | 'desc'>('asc');
	let saveQueue = Promise.resolve();

	const filtered = $derived(data.filter((item) =>
		item.name.toLowerCase().includes(search.trim().toLowerCase()),
	));
	const sorted = $derived.by(() => {
		const items = [...filtered];
		return items.sort((a, b) => {
			if (sortColumn === 'name') {
				return (sortDirection === 'asc' ? 1 : -1) * a.name.localeCompare(b.name);
			}
			const first = sortColumn === 'quantity' ? a.quantity : sortColumn === 'median' ? a.marketMedian : a.ducats;
			const second = sortColumn === 'quantity' ? b.quantity : sortColumn === 'median' ? b.marketMedian : b.ducats;
			if (first == null) return second == null ? a.name.localeCompare(b.name) : 1;
			if (second == null) return -1;
			return (sortDirection === 'asc' ? first - second : second - first) || a.name.localeCompare(b.name);
		});
	});
	const totalPlatinum = $derived(data.reduce((sum, item) => sum + (item.marketMedian ?? 0) * item.quantity, 0));
	const totalDucats = $derived(data.reduce((sum, item) => sum + (item.ducats ?? 0) * item.quantity, 0));
	const platinumFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });

	onMount(() => {
		let unlisten: UnlistenFn | undefined;
		let disposed = false;
		(async () => {
			try {
				data = (await store.get<InventoryItem[]>('items')) ?? [];
				if (disposed) return;
				if (data.some((item) => !item.slug)) void restoreLegacySlugs(() => disposed);
				unlisten = await listen<{ words: InventoryOcrWord[]; is_inventory_add?: boolean }>('ocr_result', (event) => {
					if (!event.payload.is_inventory_add) return;
					let addedAny = false;
					for (const word of event.payload.words) {
						if (!word.slug) continue;
						const existing = data.find((item) =>
							item.slug ? item.slug === word.slug : inventoryNameKey(item.name) === inventoryNameKey(word.text),
						);
						if (existing) {
							existing.quantity += 1;
								existing.slug ??= word.slug;
								existing.isCustom ??= word.is_custom;
							if (word.market_median != null) {
								existing.marketMedian = word.market_median;
								existing.marketMedianUsesOfferFallback = word.market_median_from_current_offers;
							}
							existing.ducats ??= word.ducats;
						} else {
							data.push({
								name: word.text,
								slug: word.slug,
								isCustom: word.is_custom,
								quantity: 1,
								marketMedian: word.market_median,
								marketMedianUsesOfferFallback: word.market_median_from_current_offers,
								ducats: word.ducats,
							});
						}
						addedAny = true;
					}
					if (addedAny) saveInventory();
				});
				if (disposed) unlisten();
			} catch (error) {
				console.error('Could not load inventory:', error);
			} finally {
				loading = false;
			}
		})();
		return () => {
			disposed = true;
			unlisten?.();
		};
	});

	async function restoreLegacySlugs(isDisposed: () => boolean) {
		try {
			const dictionary = DictionarySchema.parse(await invoke('get_market_dictionary'));
			if (isDisposed()) return;
			const slugsByName = new Map(dictionary.items.map((item) => [inventoryNameKey(item.name), item.slug]));
			let changed = false;
			for (const item of data) {
				if (item.slug) continue;
				const name = item.name.replace(/ \[(Exceptional|Flawless|Radiant)\]$/, '');
				const slug = slugsByName.get(inventoryNameKey(name));
				if (!slug) continue;
				item.slug = slug;
				changed = true;
			}
			if (changed) saveInventory();
		} catch (error) {
			console.error('Could not match older inventory items to market listings:', error);
		}
	}

	function saveInventory() {
		const items = $state.snapshot(data);
		saveQueue = saveQueue.catch(() => undefined).then(async () => {
			await store.set('items', items);
			await store.save();
		}).catch((error) => console.error('Could not save inventory:', error));
	}

	function updateQuantity(item: InventoryItem, delta: number) {
		item.quantity += delta;
		if (item.quantity <= 0) data = data.filter((entry) => entry !== item);
		saveInventory();
	}

	function setSort(key: string) {
		const column = key as SortColumn;
		if (sortColumn === column) sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		else {
			sortColumn = column;
			sortDirection = column === 'name' ? 'asc' : 'desc';
		}
	}
</script>

<div class="flex flex-col items-center gap-4 mx-auto p-8 w-full">
	<div class="flex flex-col gap-6 w-full max-w-5xl">
		<div>
			<h1 class="font-stretch-condensed font-bold text-2xl uppercase tracking-tight">Inventory</h1>
			<p class="mt-1 text-muted-foreground text-sm">Items captured with Ctrl+Shift+Home.</p>
		</div>

		<div class="bg-card/50 border border-border-secondary p-4">
			<div class="mb-3 text-muted-foreground text-xs font-semibold uppercase tracking-wider">Grand total</div>
			<div class="flex flex-wrap items-center gap-x-8 gap-y-3">
				<div class="flex items-center gap-2 text-xl font-semibold tabular-nums">
					{platinumFormatter.format(totalPlatinum)}
					<img src="/icons/platinum.png" class="size-5" alt="platinum" />
				</div>
				<div class="flex items-center gap-2 text-xl font-semibold tabular-nums">
					{totalDucats.toLocaleString()}
					<img src="/icons/ducats.png" class="size-5" alt="ducats" />
				</div>
			</div>
			<p class="mt-2 text-muted-foreground text-xs">Totals include items with known values.</p>
		</div>

		{#if loading}
			<p class="py-10 text-center text-muted-foreground">Loading inventory…</p>
		{:else}
			<div class="w-full">
				<label for="inventory-search" class="block mb-1.5 font-semibold text-muted-foreground text-xs">Search items</label>
				<input
					id="inventory-search"
					type="search"
					bind:value={search}
					placeholder="Search for an item..."
					class="bg-background p-2 border focus-visible:border-accent outline-none w-full text-foreground placeholder:text-muted-foreground"
				/>
			</div>
			<Table {columns} {sortColumn} {sortDirection} onSort={setSort} minWidth="920px">
				{#each sorted as item (item.slug ?? item.name)}
					<InventoryRow {item} onChangeQuantity={updateQuantity} {onOpenMarket} />
				{:else}
					<tr>
						<td colspan={columns.length} class="px-4 py-10 text-muted-foreground text-center">
							{#if data.length === 0}
								Your inventory is empty. Press Ctrl+Shift+Home in Warframe to scan items.
							{:else}
								No inventory items match this search.
							{/if}
						</td>
					</tr>
				{/each}
			</Table>
			<p class="text-muted-foreground text-sm">Showing {sorted.length} of {data.length} items</p>
		{/if}
	</div>
</div>
