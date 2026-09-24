<script lang="ts">
	import { LazyStore } from '@tauri-apps/plugin-store';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import Table, { type TableColumn } from '$lib/components/Table.svelte';
	import Button from '$lib/components/Button.svelte';
	import Combobox from '$lib/components/Combobox.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import {
		inventoryMarketSlug,
		inventoryNameKey,
		trackInventorySave,
		type InventoryItem,
	} from '$lib/inventory';
	import { mastery } from '$lib/mastery.svelte';
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
		{ key: 'totalPrice', label: 'Total P.', align: 'right', class: 'w-28' },
		{ key: 'totalDucats', label: 'Total d.', align: 'right', class: 'w-28' },
		{ key: 'links', label: 'Links', align: 'right', class: 'w-32' },
	];
	type SortColumn = 'name' | 'quantity' | 'median' | 'ducats';
	let data = $state<InventoryItem[]>([]);
	let newSlugs = $state<string[]>([]);
	let loading = $state(true);
	let search = $state('');
	let sortColumn = $state<SortColumn>('name');
	let sortDirection = $state<'asc' | 'desc'>('asc');
	let saveQueue = Promise.resolve();
	let addOpen = $state(false);
	let addItems = $state<{ label: string; value: string; ducats?: number }[]>([]);
	let addItemsAttempted = $state(false);
	let addItemsLoading = $state(false);
	let addItemsError = $state(false);
	let addPrices = $state<Record<string, { median: number; from_current_offers: boolean }>>({});
	let selectedAddSlug = $state('');
	let addQuantity = $state(1);
	const selectedAddItem = $derived(addItems.find((item) => item.value === selectedAddSlug));
	const canAddItem = $derived(
		!!selectedAddItem && Number.isSafeInteger(addQuantity) && addQuantity > 0,
	);
	$effect(() => {
		if (addOpen && !addItemsAttempted) void loadAddItems();
	});

	async function loadAddItems() {
		addItemsAttempted = true;
		addItemsLoading = true;
		addItemsError = false;
		try {
			const [dictionaryResponse, prices] = await Promise.all([
				invoke('get_market_dictionary'),
				invoke<Record<string, { median: number; from_current_offers: boolean }>>(
					'get_mastery_tradeable_prices',
				).catch((error) => {
					console.error('Could not load cached market prices:', error);
					return {};
				}),
			]);
			const dictionary = DictionarySchema.parse(dictionaryResponse);
			addItems = dictionary.items.map((item) => ({
				label: item.name,
				value: item.slug,
				ducats: item.ducats,
			}));
			addPrices = prices;
		} catch (error) {
			console.error('Could not load items for inventory:', error);
			addItemsError = true;
		} finally {
			addItemsLoading = false;
		}
	}

	function addSelectedItem() {
		if (!canAddItem || !selectedAddItem) return;
		const slug = selectedAddItem.value;
		const price = addPrices[slug] ?? mastery.prices[slug];
		const existing = data.find(
			(item) =>
				!item.isCustom &&
				(item.slug === slug ||
					(!item.slug && inventoryNameKey(item.name) === inventoryNameKey(selectedAddItem.label))),
		);
		if (existing) {
			existing.quantity += addQuantity;
			existing.slug ??= slug;
			if (price && existing.marketMedian == null) {
				existing.marketMedian = price.median;
				existing.marketMedianUsesOfferFallback = price.from_current_offers;
			}
			existing.ducats ??= selectedAddItem.ducats;
		} else {
			data.push({
				name: selectedAddItem.label,
				slug,
				quantity: addQuantity,
				marketMedian: price?.median,
				marketMedianUsesOfferFallback: price?.from_current_offers,
				ducats: selectedAddItem.ducats,
			});
		}
		saveInventory();
		selectedAddSlug = '';
		addQuantity = 1;
		addOpen = false;
	}

	const filtered = $derived(
		data.filter((item) => item.name.toLowerCase().includes(search.trim().toLowerCase())),
	);
	const sorted = $derived.by(() => {
		const items = [...filtered];
		return items.sort((a, b) => {
			if (sortColumn === 'name') {
				return (sortDirection === 'asc' ? 1 : -1) * a.name.localeCompare(b.name);
			}
			const first =
				sortColumn === 'quantity'
					? a.quantity
					: sortColumn === 'median'
						? a.marketMedian
						: a.ducats;
			const second =
				sortColumn === 'quantity'
					? b.quantity
					: sortColumn === 'median'
						? b.marketMedian
						: b.ducats;
			if (first == null) return second == null ? a.name.localeCompare(b.name) : 1;
			if (second == null) return -1;
			return (
				(sortDirection === 'asc' ? first - second : second - first) || a.name.localeCompare(b.name)
			);
		});
	});
	const totalPlatinum = $derived(
		data.reduce((sum, item) => sum + (item.marketMedian ?? 0) * item.quantity, 0),
	);
	const totalDucats = $derived(
		data.reduce((sum, item) => sum + (item.ducats ?? 0) * item.quantity, 0),
	);
	const platinumFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });
	const masteredItems = $derived.by(() => {
		const checked = new Set(mastery.checked);
		const slugs = new Set<string>();
		const names = new Set<string>();
		for (const item of mastery.items) {
			if (checked.has(item.key)) {
				if (item.marketSlug) slugs.add(item.marketSlug);
				names.add(inventoryNameKey(item.name));
			}
			for (const component of item.components) {
				if (!checked.has(item.key) && !checked.has(component.key)) continue;
				if (component.marketSlug) slugs.add(component.marketSlug);
				names.add(inventoryNameKey(`${item.name} ${component.name}`));
			}
		}
		return { slugs, names };
	});

	function isMastered(item: InventoryItem) {
		if (item.isCustom) return false;
		const slug = inventoryMarketSlug(item);
		return slug
			? masteredItems.slugs.has(slug)
			: masteredItems.names.has(inventoryNameKey(item.name));
	}

	onMount(() => {
		let unlisten: UnlistenFn | undefined;
		let disposed = false;
		(async () => {
			try {
				const [items, savedNewSlugs] = await Promise.all([
					store.get<InventoryItem[]>('items'),
					store.get<string[]>('newSlugs'),
				]);
				if (disposed) return;
				data = items ?? [];
				newSlugs = Array.isArray(savedNewSlugs) ? savedNewSlugs : [];
				if (data.some((item) => !item.slug)) void restoreLegacySlugs(() => disposed);
				unlisten = await listen<{ words: InventoryOcrWord[]; is_inventory_add?: boolean }>(
					'ocr_result',
					(event) => {
						if (!event.payload.is_inventory_add) return;
						let addedAny = false;
						for (const word of event.payload.words) {
							if (!word.slug) continue;
							if (!newSlugs.includes(word.slug)) newSlugs.push(word.slug);
							const existing = data.find((item) =>
								item.slug
									? item.slug === word.slug
									: inventoryNameKey(item.name) === inventoryNameKey(word.text),
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
					},
				);
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
			const slugsByName = new Map(
				dictionary.items.map((item) => [inventoryNameKey(item.name), item.slug]),
			);
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
		const savedNewSlugs = [...newSlugs];
		saveQueue = saveQueue
			.catch(() => undefined)
			.then(async () => {
				await store.set('items', items);
				await store.set('newSlugs', savedNewSlugs);
				await store.save();
			})
			.catch((error) => console.error('Could not save inventory:', error));
		trackInventorySave(saveQueue);
	}

	function updateQuantity(item: InventoryItem, delta: number) {
		item.quantity += delta;
		if (item.quantity <= 0) {
			data = data.filter((entry) => entry !== item);
			if (item.slug) newSlugs = newSlugs.filter((slug) => slug !== item.slug);
		}
		saveInventory();
	}

	function dismissNewDots() {
		newSlugs = [];
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
		<div class="bg-card/50 p-4 border border-border-secondary">
			<div class="mb-3 font-semibold text-muted-foreground text-xs uppercase tracking-wider">
				Grand total
			</div>
			<div class="flex flex-wrap items-center gap-x-8 gap-y-3">
				<div class="flex items-center gap-2 font-semibold tabular-nums text-xl">
					{platinumFormatter.format(totalPlatinum)}
					<img src="/icons/platinum.png" class="size-5" alt="platinum" />
				</div>
				<div class="flex items-center gap-2 font-semibold tabular-nums text-xl">
					{totalDucats.toLocaleString()}
					<img src="/icons/ducats.png" class="size-5" alt="ducats" />
				</div>
			</div>
			<p class="mt-2 text-muted-foreground text-xs">Totals include items with known values.</p>
			{#if newSlugs.length > 0}
				<Button onclick={dismissNewDots} class="mt-3 text-xs">
					Dismiss {newSlugs.length} new dots
				</Button>
			{/if}
		</div>

		{#if loading}
			<p class="py-10 text-muted-foreground text-center">Loading inventory…</p>
		{:else}
			<div class="w-full">
				<label
					for="inventory-search"
					class="block mb-1.5 font-semibold text-muted-foreground text-xs"
				>
					Search items
				</label>
				<div class="flex items-center gap-3">
					<input
						id="inventory-search"
						type="search"
						bind:value={search}
						placeholder="Search for an item..."
						class="flex-1 bg-background p-2 border focus-visible:border-accent outline-none min-w-0 h-10 text-foreground placeholder:text-muted-foreground"
					/>
					<Button
						onclick={() => (addOpen = true)}
						class="flex items-center gap-1 h-10 text-xs shrink-0"
					>
						<Icon icon="lucide:plus" class="size-3.5" /> Add item
					</Button>
				</div>
			</div>
			<Table {columns} {sortColumn} {sortDirection} onSort={setSort} minWidth="920px">
				{#each sorted as item (item.slug ?? item.name)}
					<InventoryRow
						{item}
						mastered={isMastered(item)}
						isNew={!!item.slug && newSlugs.includes(item.slug)}
						onChangeQuantity={updateQuantity}
						{onOpenMarket}
					/>
				{:else}
					<tr>
						<td colspan={columns.length} class="px-4 py-10 text-muted-foreground text-center">
							{#if data.length === 0}
								Your inventory is empty.
							{:else}
								No inventory items match this search.
							{/if}
						</td>
					</tr>
				{/each}
			</Table>
			<p class="text-muted-foreground text-sm">Showing {sorted.length} of {data.length} items</p>
			{#snippet addTitle()}Add inventory item{/snippet}
			{#snippet addDescription()}Search the market item list and add it to your inventory.{/snippet}
			{#snippet addClose()}<Button>Cancel</Button>{/snippet}
			{#snippet addActions()}
				<Button variant="primary" disabled={!canAddItem} onclick={addSelectedItem}>
					Add to inventory
				</Button>
			{/snippet}
			<Dialog
				bind:open={addOpen}
				title={addTitle}
				description={addDescription}
				dialogClose={addClose}
				dialogActions={addActions}
				contentProps={{ class: 'h-auto' }}
			>
				<div class="flex items-start gap-3 px-6 pb-2">
					<div class="flex-1 min-w-0">
						<label
							for="inventory-add-item"
							class="block mb-1.5 font-semibold text-muted-foreground text-xs"
						>
							Item
						</label>
						<Combobox
							type="single"
							items={addItems}
							bind:value={selectedAddSlug}
							inputValue={selectedAddItem?.label ?? ''}
							disabled={addItemsLoading || addItemsError}
							inputProps={{
								id: 'inventory-add-item',
								placeholder: addItemsLoading ? 'Loading items...' : 'Search for an item...',
							}}
						/>
						{#if addItemsError}
							<div role="alert" class="flex items-center gap-2 mt-2 text-sm">
								<span>Could not load the item list.</span>
								<button class="underline cursor-pointer" onclick={loadAddItems}>Retry</button>
							</div>
						{/if}
					</div>
					<div class="w-28 shrink-0">
						<label
							for="inventory-add-quantity"
							class="block mb-1.5 font-semibold text-muted-foreground text-xs"
						>
							Quantity
						</label>
						<input
							id="inventory-add-quantity"
							type="number"
							min="1"
							step="1"
							bind:value={addQuantity}
							class="bg-background p-2 border focus-visible:border-accent outline-none w-full text-foreground"
						/>
					</div>
				</div>
			</Dialog>
		{/if}
	</div>
</div>
