<script lang="ts">
	import Currency from '$lib/components/Currency.svelte';
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
		addInventoryItem,
		changeInventoryRowQuantity,
		dismissNewInventoryItems,
		type InventoryItem,
	} from '$lib/inventory';
	import { mastery } from '$lib/mastery.svelte';
	import { inventory, reloadInventory } from '$lib/inventory.svelte';
	import {
		warframeItems,
		refreshWarframeItemCatalog,
		refreshWarframeItemListings,
		applyWarframeListingChange,
	} from '$lib/warframe-item.svelte';
	import { formatWfmTag, wfmCategory } from '$lib/wfm-tags';
	import InventoryRow from './InventoryRow.svelte';
	import CreateListing from './CreateListing.svelte';
	import { masteredMarketItems } from '$lib/listing-context';
	import { marketAccount } from '$lib/market-account.svelte';
	import type { Listing } from '../listings/types';
	import Tooltip from '$lib/components/Tooltip.svelte';

	let { onOpenMarket = () => {} }: { onOpenMarket?: (slug: string) => void } = $props();

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
	const data = $derived(inventory.items);
	const newSlugs = $derived(inventory.newSlugs);
	const loading = $derived(!inventory.ready);
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
	const filterControls = $derived.by(() => [
		{
			id: 'inventory-category',
			label: 'Category',
			options: categoryOptions,
			value: categoryFilter,
			setValue: (value: string) => (categoryFilter = value),
		},
		{
			id: 'inventory-tag',
			label: 'Tag',
			options: tagOptions,
			value: tagFilter,
			setValue: (value: string) => (tagFilter = value),
		},
		{
			id: 'inventory-mastery',
			label: 'Mastery',
			options: masteryOptions,
			value: masteryFilter,
			setValue: (value: string) => (masteryFilter = value),
		},
		{
			id: 'inventory-listing',
			label: 'Listing',
			options: listingOptions,
			value: listingFilter,
			setValue: (value: string) => (listingFilter = value),
			disabled: !listingsLoaded,
		},
	]);
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
	const listings = $derived(warframeItems.listings);
	const listingsLoaded = $derived(warframeItems.listingsLoaded);
	const listingsError = $derived(warframeItems.listingsError);
	const listingBySlug = $derived.by(() => {
		const bySlug = new Map<string, Listing>();
		for (const listing of listings) {
			if (listing.type !== 'sell') continue;
			const slug = warframeItems.byId[listing.itemId]?.slug;
			if (slug && !bySlug.has(slug)) bySlug.set(slug, listing);
		}
		return bySlug;
	});
	const addItems = $derived(warframeItems.addOptions);
	const addItemsLoading = $derived(!warframeItems.identitiesReady);
	const addItemsError = $derived(warframeItems.identitiesError);
	let selectedAddSlug = $state('');
	let addQuantity = $state(1);
	const selectedAddItem = $derived(addItems.find((item) => item.value === selectedAddSlug));
	const canAddItem = $derived(
		inventory.ready && !!selectedAddItem && Number.isSafeInteger(addQuantity) && addQuantity > 0,
	);
	function metadataFor(item: InventoryItem) {
		const slug = inventoryMarketSlug(item);
		return slug
			? (warframeItems.bySlug[slug] ?? warframeItems.bySlug[slug.replace(/_rank_\d+$/, '')])
			: undefined;
	}

	function categoryFor(item: InventoryItem): string {
		const metadata = metadataFor(item);
		return metadata ? wfmCategory(metadata.tags) : item.category ?? 'Other';
	}

	function addSelectedItem() {
		if (!canAddItem || !selectedAddItem) return;
		void addInventoryItem({ text: selectedAddItem.label, slug: selectedAddItem.value, ducats: selectedAddItem.ducats }, addQuantity)
			.catch((error) => console.error('Could not save inventory:', error));
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

	function openListing(item: InventoryItem, listing: Listing | null) {
		editingListing = listing;
		listingItem = item;
	}

	function updateQuantity(item: InventoryItem, delta: number) {
		void changeInventoryRowQuantity(item, delta)
			.catch((error) => console.error('Could not save inventory:', error));
	}

	function dismissNewDots() {
		void dismissNewInventoryItems()
			.catch((error) => console.error('Could not save inventory:', error));
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

<div class="mx-auto p-8 w-full max-w-5xl page-width">
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
						<Currency value={totalPlatinum} currency="platinum" iconClass="size-4" />
					</div>
					<div class="flex items-center gap-1.5 pl-4 tabular-nums">
						<Currency value={totalDucats} currency="ducats" iconClass="size-4" />
					</div>
				</section>
			</div>
			<div class="flex flex-wrap items-center gap-2">
				{#if newSlugs.length > 0}<Button onclick={dismissNewDots} class="text-base">
						Dismiss {newSlugs.length} new dots
					</Button>{/if}
				<Button
					variant="primary"
					disabled={!inventory.ready}
					onclick={() => (addOpen = true)}
					class="inline-flex items-center gap-1.5 text-base"
				>
					<Icon icon="lucide:plus" class="size-4" /> Add item
				</Button>
			</div>
		</header>
		<div class="bg-surface my-1 w-full h-px"></div>
		{#if inventory.error}
			<p role="alert" class="text-danger text-base">
				{inventory.error}
				<button type="button" class="underline cursor-pointer" onclick={reloadInventory}>Retry</button>
			</p>
		{/if}
		{#if listingsError && marketAccount.session}
			<p role="alert" class="text-danger text-base">
				Could not load your listings. <button
					type="button"
					class="underline cursor-pointer"
					onclick={refreshWarframeItemListings}
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
					class="block mb-1.5 font-semibold text-muted-foreground text-sm"
				>
					Search items
				</label>
				<input
					id="inventory-search"
					data-item-search
					type="search"
					bind:value={search}
					placeholder="Search for an item..."
					class="bg-background p-2 border focus-visible:border-accent outline-none w-full h-10 text-foreground placeholder:text-muted-foreground"
				/>
			</div>
			{#each filterControls as filter (filter.id)}
				<div class="min-w-0">
					<label for={filter.id} class="block mb-1.5 font-semibold text-muted-foreground text-sm">
						{filter.label}
					</label>
					<Select
						type="single"
						items={filter.options}
						bind:value={() => filter.value, filter.setValue}
						disabled={filter.disabled ?? false}
						triggerProps={{ id: filter.id, class: 'max-w-none h-10' }}
					/>
				</div>
			{/each}
		</div>
		{#if addItemsError}
			<p role="alert" class="text-danger text-base">
				Could not load item categories and tags. <button
					type="button"
					class="underline cursor-pointer"
					onclick={refreshWarframeItemCatalog}
				>
					Retry
				</button>
			</p>
		{/if}
		{#snippet inventoryRow(item: InventoryItem, index: number, measureRow: (element: HTMLTableRowElement) => void)}
			{@const slug = inventoryMarketSlug(item)}
			<InventoryRow
				virtualIndex={index}
				{measureRow}
				{item}
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
			virtualize
			emptyMessage={loading
				? 'Loading inventory…'
				: inventory.error || (data.length === 0
					? 'Your inventory is empty.'
					: 'No inventory items match these filters.')}
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
			onSaved={applyWarframeListingChange}
		/>
		<p class="text-muted-foreground text-base">Showing {sorted.length} of {data.length} items</p>
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
						class="block mb-1.5 font-semibold text-muted-foreground text-sm"
					>
						Item
					</label>
					<Combobox
						type="single"
						items={addItems}
						bind:value={selectedAddSlug}
						inputValue={selectedAddItem?.label ?? ''}
						disabled={addItemsLoading || !!addItemsError || !inventory.ready}
						inputProps={{ id: 'inventory-add-item', placeholder: 'Search for an item...' }}
					/>
					{#if addItemsError}
						<div role="alert" class="flex items-center gap-2 mt-2 text-base">
							<span>Could not load the item list.</span>
							<button class="underline cursor-pointer" onclick={refreshWarframeItemCatalog}>Retry</button>
						</div>
					{/if}
				</div>
				<div class="w-28 shrink-0">
					<label
						for="inventory-add-quantity"
						class="block mb-1.5 font-semibold text-muted-foreground text-sm"
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
	</div>
</div>
