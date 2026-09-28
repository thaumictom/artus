<script lang="ts">
	import { LazyStore } from '@tauri-apps/plugin-store';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { onMount, untrack } from 'svelte';
	import Icon from '@iconify/svelte';
	import Table from '$lib/components/Table.svelte';
	import type { TableColumn } from '$lib/components/table-types';
	import Button from '$lib/components/Button.svelte';
	import Select from '$lib/components/Select.svelte';
	import Combobox from '$lib/components/Combobox.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import {
		inventoryMarketSlug,
		inventoryNameKey,
		trackInventorySave,
		waitForInventorySave,
		type InventoryItem,
	} from '$lib/inventory';
	import { mastery } from '$lib/mastery.svelte';
	import { DictionarySchema } from '$lib/schemas';
	import { formatWfmTag, wfmCategory } from '$lib/wfm-tags';
	import InventoryRow from './InventoryRow.svelte';
	import CreateListing from './CreateListing.svelte';
	import { masteredMarketItems } from '$lib/listing-context';
	import { marketAccount } from '$lib/market-account.svelte';
	import { fetchMarketListings } from '$lib/market-listings';
	import type { Listing, ListingChange, ListingItem } from '../listings/types';
	import Tooltip from '$lib/components/Tooltip.svelte';

	let { onOpenMarket = () => {} }: { onOpenMarket?: (slug: string) => void } = $props();

	const store = new LazyStore('inventory.json');
	const columns: TableColumn[] = [
		{ key: 'name', label: 'Item', sortable: true, class: 'min-w-48' },
		{ key: 'quantity', label: 'Quantity', sortable: true, align: 'right', class: 'w-36' },
		{ key: 'median', label: 'Median', sortable: true, align: 'right', class: 'w-24' },
		{ key: 'ducats', label: 'Ducats', sortable: true, align: 'right', class: 'w-24' },
		{ key: 'totalPrice', label: 'Total P.', align: 'right', class: 'w-28' },
		{ key: 'totalDucats', label: 'Total d.', align: 'right', class: 'w-28' },
		{ key: 'links', label: '', align: 'right', class: 'w-0' },
	];
	type SortColumn = 'name' | 'quantity' | 'median' | 'ducats';
	let data = $state<InventoryItem[]>([]);
	let newSlugs = $state<string[]>([]);
	let loading = $state(true);
	let search = $state('');
	let categoryFilter = $state('All');
	let tagFilter = $state('All');
	let masteryFilter = $state('all');
	let listingFilter = $state('all');
	const masteryOptions = [
		{ value: 'all', label: 'All items' },
		{ value: 'mastered', label: 'Mastered' },
		{ value: 'unmastered', label: 'Not mastered' },
	];
	const listingOptions = [
		{ value: 'all', label: 'All items' },
		{ value: 'listed', label: 'Listed' },
		{ value: 'unlisted', label: 'Not listed' },
	];
	type ItemMetadata = { category: string; tags: string[] };
	let metadataBySlug = $state.raw<Record<string, ItemMetadata>>({});
	let metadataRequest = 0;
	const categories = $derived(['All', ...new Set(data.map((item) => categoryFor(item)).sort())]);
	const tags = $derived([
		'All',
		...new Set(data.flatMap((item) => metadataFor(item)?.tags ?? []).sort()),
	]);
	const categoryOptions = $derived(categories.map((value) => ({ value, label: value })));
	const tagOptions = $derived(
		tags.map((value) => ({ value, label: value === 'All' ? value : formatWfmTag(value) })),
	);
	let sortColumn = $state<SortColumn>('name');
	let sortDirection = $state<'asc' | 'desc'>('asc');
	let addOpen = $state(false);
	let listingItem = $state<InventoryItem | null>(null);
	let editingListing = $state<Listing | null>(null);
	let listings = $state<Listing[]>([]);
	let listingDetails = $state<Record<string, ListingItem>>({});
	let listingsLoaded = $state(false);
	let listingsError = $state<string | null>(null);
	let listingsRequest = 0;
	const listingBySlug = $derived.by(() => {
		const bySlug = new Map<string, Listing>();
		for (const listing of listings) {
			if (listing.type !== 'sell') continue;
			const slug = listingDetails[listing.itemId]?.slug;
			if (slug && !bySlug.has(slug)) bySlug.set(slug, listing);
		}
		return bySlug;
	});
	let addItems = $state<{ label: string; value: string; ducats?: number }[]>([]);
	let addItemsLoading = $state(true);
	let addItemsError = $state(false);
	let selectedAddSlug = $state('');
	let addQuantity = $state(1);
	const selectedAddItem = $derived(addItems.find((item) => item.value === selectedAddSlug));
	const canAddItem = $derived(
		!!selectedAddItem && Number.isSafeInteger(addQuantity) && addQuantity > 0,
	);
	async function loadAddItems() {
		const request = ++metadataRequest;
		addItemsLoading = true;
		addItemsError = false;
		try {
			const dictionaryResponse = await invoke('get_market_dictionary');
			const dictionary = DictionarySchema.parse(dictionaryResponse);
			if (request !== metadataRequest) return;
			addItems = dictionary.items.map((item) => ({
				label: item.name,
				value: item.slug,
				ducats: item.ducats,
			}));
			metadataBySlug = Object.fromEntries(
				dictionary.items.map((item) => [
					item.slug,
					{ category: wfmCategory(item.tags), tags: item.tags },
				]),
			);
		} catch (error) {
			console.error('Could not load items for inventory:', error);
			if (request === metadataRequest && addItems.length === 0) addItemsError = true;
		} finally {
			if (request === metadataRequest) addItemsLoading = false;
		}
	}

	function metadataFor(item: InventoryItem): ItemMetadata | undefined {
		const slug = inventoryMarketSlug(item);
		return slug
			? (metadataBySlug[slug] ?? metadataBySlug[slug.replace(/_rank_\d+$/, '')])
			: undefined;
	}

	function categoryFor(item: InventoryItem): string {
		return metadataFor(item)?.category ?? item.category ?? 'Other';
	}

	function addSelectedItem() {
		if (!canAddItem || !selectedAddItem) return;
		const slug = selectedAddItem.value;
		const existing = data.find(
			(item) =>
				!item.isCustom &&
				(item.slug === slug ||
					(!item.slug && inventoryNameKey(item.name) === inventoryNameKey(selectedAddItem.label))),
		);
		if (existing) {
			existing.quantity += addQuantity;
			existing.slug ??= slug;
			existing.ducats ??= selectedAddItem.ducats;
		} else {
			data.push({
				name: selectedAddItem.label,
				slug,
				quantity: addQuantity,
				ducats: selectedAddItem.ducats,
			});
		}
		saveInventory();
		selectedAddSlug = '';
		addQuantity = 1;
		addOpen = false;
	}

	const filtered = $derived(
		data.filter((item) => {
			if (!item.name.toLowerCase().includes(search.trim().toLowerCase())) return false;
			if (categoryFilter !== 'All' && categoryFor(item) !== categoryFilter) return false;
			if (tagFilter !== 'All' && !metadataFor(item)?.tags.includes(tagFilter)) return false;
			if (masteryFilter !== 'all' && isMastered(item) !== (masteryFilter === 'mastered'))
				return false;
			if (listingFilter !== 'all' && listingsLoaded) {
				const slug = inventoryMarketSlug(item);
				const listed = !!slug && listingBySlug.has(slug);
				if (listed !== (listingFilter === 'listed')) return false;
			}
			return true;
		}),
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
						? priceFor(a)?.median
						: a.ducats;
			const second =
				sortColumn === 'quantity'
					? b.quantity
					: sortColumn === 'median'
						? priceFor(b)?.median
						: b.ducats;
			if (first == null) return second == null ? a.name.localeCompare(b.name) : 1;
			if (second == null) return -1;
			return (
				(sortDirection === 'asc' ? first - second : second - first) || a.name.localeCompare(b.name)
			);
		});
	});
	const totalPlatinum = $derived(
		data.reduce((sum, item) => sum + (priceFor(item)?.median ?? 0) * item.quantity, 0),
	);
	const totalDucats = $derived(
		data.reduce((sum, item) => sum + (item.ducats ?? 0) * item.quantity, 0),
	);
	const platinumFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });
	const masteredItems = $derived(masteredMarketItems(mastery.items, mastery.checked));
	function priceFor(item: InventoryItem) {
		return item.slug ? mastery.prices[item.slug] : undefined;
	}

	function isMastered(item: InventoryItem) {
		if (item.isCustom) return false;
		const slug = inventoryMarketSlug(item);
		return slug
			? masteredItems.slugs.has(slug)
			: masteredItems.names.has(inventoryNameKey(item.name));
	}

	async function refreshListings(accountName: string) {
		const request = ++listingsRequest;
		listingsLoaded = false;
		listingsError = null;
		try {
			const [orders, details] = await Promise.all([
				fetchMarketListings(),
				Object.keys(listingDetails).length
					? Promise.resolve(listingDetails)
					: invoke<Record<string, ListingItem>>('market_item_details'),
			]);
			if (request !== listingsRequest || marketAccount.session?.ingameName !== accountName) return;
			listings = orders;
			listingDetails = details;
			listingsLoaded = true;
		} catch (error) {
			if (request === listingsRequest) listingsError = String(error);
		}
	}

	$effect(() => {
		const accountName = marketAccount.session?.ingameName;
		if (accountName) untrack(() => void refreshListings(accountName));
		else {
			++listingsRequest;
			listings = [];
			listingsLoaded = false;
			listingsError = null;
		}
	});

	function openListing(item: InventoryItem, listing: Listing | null) {
		editingListing = listing;
		listingItem = item;
	}

	function applyListingChange(change: ListingChange) {
		switch (change.kind) {
			case 'created':
				if (change.listing) listings = [...listings, change.listing];
				else if (marketAccount.session) void refreshListings(marketAccount.session.ingameName);
				break;
			case 'updated':
				listings = listings.map((listing) =>
					listing.id === change.id
						? { ...listing, platinum: change.platinum, quantity: change.quantity }
						: listing,
				);
				break;
			case 'visibility':
				listings = listings.map((listing) =>
					listing.id === change.id ? { ...listing, visible: change.visible } : listing,
				);
				break;
			case 'deleted':
				listings = listings.filter((listing) => listing.id !== change.id);
				break;
		}
	}

	onMount(() => {
		let unlisten: (() => void) | undefined;
		let disposed = false;
		(async () => {
			try {
				unlisten = await store.onChange<unknown>((key, value) => {
					if (key === 'items' && Array.isArray(value)) data = value as InventoryItem[];
					if (key === 'newSlugs' && Array.isArray(value)) newSlugs = value as string[];
				});
				if (disposed) {
					unlisten();
					return;
				}
				await waitForInventorySave().catch(() => undefined);
				const [items, savedNewSlugs] = await Promise.all([
					store.get<InventoryItem[]>('items'),
					store.get<string[]>('newSlugs'),
				]);
				if (disposed) return;
				data = items ?? [];
				newSlugs = Array.isArray(savedNewSlugs) ? savedNewSlugs : [];
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

	onMount(() => {
		let disposed = false;
		let unlistenCatalog: (() => void) | undefined;
		void loadAddItems();
		void listen('api_catalogs_fetched', () => {
			void loadAddItems();
		})
			.then((cleanup) => {
				if (disposed) cleanup();
				else unlistenCatalog = cleanup;
			})
			.catch((error) => console.error('Could not observe item catalog refresh:', error));
		return () => {
			disposed = true;
			metadataRequest++;
			unlistenCatalog?.();
		};
	});

	function saveInventory() {
		const items = $state.snapshot(data);
		const savedNewSlugs = [...newSlugs];
		const save = waitForInventorySave()
			.catch(() => undefined)
			.then(async () => {
				await store.set('items', items);
				await store.set('newSlugs', savedNewSlugs);
				await store.save();
			})
			.catch((error) => console.error('Could not save inventory:', error));
		trackInventorySave(save);
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

<div class="mx-auto py-6 w-full max-w-5xl">
	<div class="flex flex-col gap-4">
		<header class="flex flex-wrap justify-between items-center gap-4 w-full">
			<div>
				<section class="flex flex-wrap items-center divide-border-secondary divide-x">
					<div class="flex items-center gap-1.5 pr-4">
						<span>{data.length} items</span>
						{#if data.length}
							<Tooltip side="bottom" align="center">
								{#snippet children()}<Icon
										icon="material-symbols:info-outline-rounded"
										class="size-4 text-muted-foreground"
									/>{/snippet}
								{#snippet content()}
									<div>Total number of items in your inventory that have a price.</div>
								{/snippet}
							</Tooltip>
						{/if}
					</div>
					<div class="flex items-center gap-1.5 px-4 tabular-nums">
						{platinumFormatter.format(totalPlatinum)}
						<img src="/icons/platinum.png" class="size-4" alt="platinum" />
					</div>
					<div class="flex items-center gap-1.5 pl-4 tabular-nums">
						{totalDucats.toLocaleString()}
						<img src="/icons/ducats.png" class="size-4" alt="ducats" />
					</div>
				</section>
			</div>
			<div class="flex flex-wrap items-center gap-2">
				{#if newSlugs.length > 0}<Button onclick={dismissNewDots} class="text-sm">
						Dismiss {newSlugs.length} new dots
					</Button>{/if}
				<Button
					variant="primary"
					onclick={() => (addOpen = true)}
					class="inline-flex items-center gap-1.5 text-sm"
				>
					<Icon icon="lucide:plus" class="size-4" /> Add item
				</Button>
			</div>
		</header>
		<div class="bg-surface my-1 w-full h-px"></div>

		{#if loading}
			<p class="py-10 text-muted-foreground text-center">Loading inventory…</p>
		{:else}
			{#if listingsError && marketAccount.session}
				<p role="alert" class="text-danger text-sm">
					Could not load your listings. <button
						type="button"
						class="underline cursor-pointer"
						onclick={() => refreshListings(marketAccount.session!.ingameName)}
					>
						Retry
					</button>
				</p>
			{/if}
			<div
				class="items-end gap-3 grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-[minmax(14rem,2fr)_minmax(9rem,1.25fr)_minmax(9rem,1.25fr)_minmax(7rem,0.75fr)_minmax(7rem,0.75fr)]"
			>
				<div class="sm:col-span-2 xl:col-span-1 min-w-0">
					<label
						for="inventory-search"
						class="block mb-1.5 font-semibold text-muted-foreground text-xs"
					>
						Search items
					</label>
					<input
						id="inventory-search"
						type="search"
						bind:value={search}
						placeholder="Search for an item..."
						class="bg-background p-2 border focus-visible:border-accent outline-none w-full h-10 text-foreground placeholder:text-muted-foreground"
					/>
				</div>
				<div class="min-w-0">
					<label
						for="inventory-category"
						class="block mb-1.5 font-semibold text-muted-foreground text-xs"
					>
						Category
					</label>
					<Select
						type="single"
						items={categoryOptions}
						bind:value={categoryFilter}
						triggerProps={{ id: 'inventory-category', class: 'max-w-none h-10' }}
					/>
				</div>
				<div class="min-w-0">
					<label
						for="inventory-tag"
						class="block mb-1.5 font-semibold text-muted-foreground text-xs"
					>
						Tag
					</label>
					<Select
						type="single"
						items={tagOptions}
						bind:value={tagFilter}
						triggerProps={{ id: 'inventory-tag', class: 'max-w-none h-10' }}
					/>
				</div>
				<div class="min-w-0">
					<label
						for="inventory-mastery"
						class="block mb-1.5 font-semibold text-muted-foreground text-xs"
					>
						Mastery
					</label>
					<Select
						type="single"
						items={masteryOptions}
						bind:value={masteryFilter}
						triggerProps={{ id: 'inventory-mastery', class: 'max-w-none h-10' }}
					/>
				</div>
				<div class="min-w-0">
					<label
						for="inventory-listing"
						class="block mb-1.5 font-semibold text-muted-foreground text-xs"
					>
						Listing
					</label>
					<Select
						type="single"
						items={listingOptions}
						bind:value={listingFilter}
						disabled={!listingsLoaded}
						triggerProps={{ id: 'inventory-listing', class: 'max-w-none h-10' }}
					/>
				</div>
			</div>
			{#if addItemsError}
				<p role="alert" class="text-danger text-sm">
					Could not load item categories and tags. <button
						type="button"
						class="underline cursor-pointer"
						onclick={loadAddItems}
					>
						Retry
					</button>
				</p>
			{/if}
			{#snippet inventoryRow(item: InventoryItem)}
				{@const slug = inventoryMarketSlug(item)}
				<InventoryRow
					{item}
					category={categoryFor(item)}
					tags={metadataFor(item)?.tags ?? []}
					price={priceFor(item)}
					mastered={isMastered(item)}
					listing={slug ? listingBySlug.get(slug) : undefined}
					{listingsLoaded}
					isNew={!!item.slug && newSlugs.includes(item.slug)}
					onChangeQuantity={updateQuantity}
					{onOpenMarket}
					onOpenListing={openListing}
				/>
			{/snippet}
			<Table
				{columns}
				rows={sorted}
				rowKey={(item) => item.slug ?? item.name}
				renderRow={inventoryRow}
				emptyMessage={data.length === 0
					? 'Your inventory is empty.'
					: 'No inventory items match these filters.'}
				{sortColumn}
				{sortDirection}
				onSort={setSort}
				minWidth="920px"
			/>
			<CreateListing
				bind:item={
					() => listingItem,
					(value) => {
						listingItem = value;
						if (value === null) editingListing = null;
					}
				}
				editing={editingListing}
				mastered={listingItem ? isMastered(listingItem) : false}
				onSaved={applyListingChange}
			/>
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
