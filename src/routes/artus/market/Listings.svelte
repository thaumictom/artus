<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount, tick } from 'svelte';
	import { LazyStore } from '@tauri-apps/plugin-store';
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Table, { type TableColumn } from '$lib/components/Table.svelte';
	import ActionPopover from '$lib/components/ActionPopover.svelte';
	import Combobox from '$lib/components/Combobox.svelte';
	import ListingItemInfo from '$lib/components/ListingItemInfo.svelte';
	import ListingQuantityWarning from '$lib/components/ListingQuantityWarning.svelte';
	import Select from '$lib/components/Select.svelte';
	import { marketAccount, marketProfileUrl } from '$lib/market-account.svelte';
	import { removeOneMarketInventoryItem, waitForInventorySave, type InventoryItem } from '$lib/inventory';
	import { mastery } from '$lib/mastery.svelte';
	import { isMarketItemMastered, masteredMarketItems, ownedMarketCount } from '$lib/listing-context';
	import { DictionarySchema } from '$lib/schemas';
	import CreateListing from '../inventory/CreateListing.svelte';

	let { onOpenMarket = () => {} }: { onOpenMarket?: (slug: string) => void } = $props();
	type Listing = { id: string; itemId: string; type: 'buy' | 'sell'; platinum: number; quantity: number; visible: boolean; perTrade?: number; rank?: number; subtype?: string };
	type ListingItem = { name: string; slug: string; icon?: string | null };
	const inventoryStore = new LazyStore('inventory.json');
	let orders = $state<Listing[]>([]);
	let itemDetails = $state<Record<string, ListingItem>>({});
	let inventoryItems = $state<InventoryItem[]>([]);
	let loading = $state(false);
	let busy = $state(false);
	let error = $state<string | null>(null);
	let search = $state('');
	let typeFilter = $state<'all' | 'buy' | 'sell'>('all');
	let statusFilter = $state<'all' | 'visible' | 'hidden'>('all');
	let sortColumn = $state<'name' | 'price' | 'quantity' | 'status'>('name');
	let sortDirection = $state<'asc' | 'desc'>('asc');
	let loadedFor = $state<string | null>(null);
	let lastFetchedAt = $state<Date | null>(null);
	let menuOpenFor = $state<string | null>(null);
	let createPickerOpen = $state(false);
	let createItem = $state<InventoryItem | null>(null);
	let selectedCreateSlug = $state('');
	let catalogItems = $state<{ label: string; value: string }[]>([]);
	let catalogLoading = $state(false);
	let catalogError = $state<string | null>(null);
	let editing = $state<Listing | null>(null);
	let editSellPrices = $state<number[]>([]);
	let editBuyPrices = $state<number[]>([]);
	let editMarketMedian = $state<number | null>(null);
	let editMedianUsesOfferFallback = $state(false);
	let editPricesLoading = $state(false);
	let editPricesError = $state<string | null>(null);
	let editPricesRequestId = 0;
	let removing = $state<Listing | null>(null);
	let price = $state(1);
	let quantity = $state(1);
	const columns: TableColumn[] = [
		{ key: 'name', label: 'Name', sortable: true, class: 'min-w-64' },
		{ key: 'type', label: 'Type', class: 'w-24' },
		{ key: 'price', label: 'Price', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'quantity', label: 'Quantity', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'status', label: 'Status', sortable: true, class: 'w-28' },
		{ key: 'actions', label: 'Actions', align: 'right', class: 'w-40' },
	];
	const sellCount = $derived(orders.filter((order) => order.type === 'sell').length);
	const visibleCount = $derived(orders.filter((order) => order.visible).length);
	const hiddenCount = $derived(orders.length - visibleCount);
	const valid = $derived(Number.isSafeInteger(price) && price >= 1 && price <= 900000 && Number.isSafeInteger(quantity) && quantity >= 1 && quantity <= 9999);
	const masteredIndex = $derived(masteredMarketItems(mastery.items, mastery.checked));
	const detailsFor = (order: Listing) => itemDetails[order.itemId];
	const nameFor = (order: Listing) => detailsFor(order)?.name ?? order.itemId;
	const slugFor = (order: Listing) => detailsFor(order)?.slug ?? order.itemId;
	const ownedFor = (order: Listing) => ownedMarketCount(inventoryItems, detailsFor(order)?.slug, nameFor(order));
	const filteredOrders = $derived.by(() => {
		const query = search.trim().toLocaleLowerCase();
		return orders.filter((order) => {
			if (typeFilter !== 'all' && order.type !== typeFilter) return false;
			if (statusFilter !== 'all' && order.visible !== (statusFilter === 'visible')) return false;
			if (!query) return true;
			return `${nameFor(order)} ${order.itemId} ${order.type} ${order.subtype ?? ''}`
				.toLocaleLowerCase().includes(query);
		});
	});
	const sortedOrders = $derived.by(() => {
		const direction = sortDirection === 'asc' ? 1 : -1;
		return [...filteredOrders].sort((a, b) => {
			const comparison = sortColumn === 'name'
				? nameFor(a).localeCompare(nameFor(b), undefined, { sensitivity: 'base' })
				: sortColumn === 'price'
					? a.platinum - b.platinum
					: sortColumn === 'quantity'
						? a.quantity - b.quantity
						: Number(a.visible) - Number(b.visible);
			return direction * (comparison || nameFor(a).localeCompare(nameFor(b), undefined, { sensitivity: 'base' }) || a.id.localeCompare(b.id));
		});
	});
	const masteredFor = (order: Listing) => isMarketItemMastered(masteredIndex, detailsFor(order)?.slug, nameFor(order));
	const selectedCreateItem = $derived(catalogItems.find((item) => item.value === selectedCreateSlug));
	const typeFilterItems = [
		{ value: 'all', label: 'Buy and sell' },
		{ value: 'buy', label: 'Buy' },
		{ value: 'sell', label: 'Sell' },
	];
	const statusFilterItems = [
		{ value: 'all', label: 'All listings' },
		{ value: 'visible', label: 'Visible' },
		{ value: 'hidden', label: 'Hidden' },
	];
	const fetchedAtFormatter = new Intl.DateTimeFormat(undefined, { dateStyle: 'short', timeStyle: 'medium' });
	$effect(() => { if (createPickerOpen && catalogItems.length === 0 && !catalogLoading && !catalogError) void loadCatalog(); });

	async function loadCatalog() {
		catalogLoading = true; catalogError = null;
		try {
			const dictionary = DictionarySchema.parse(await invoke('get_market_dictionary'));
			catalogItems = dictionary.items.map((item) => ({ label: item.name, value: item.slug }));
		} catch (cause) { catalogError = String(cause); }
		finally { catalogLoading = false; }
	}

	async function selectCreateItem() {
		if (!selectedCreateItem) return;
		const selected = selectedCreateItem;
		createPickerOpen = false;
		selectedCreateSlug = '';
		await tick();
		requestAnimationFrame(() => {
			createItem = {
				name: selected.label, slug: selected.value,
				quantity: ownedMarketCount(inventoryItems, selected.value, selected.label),
			};
		});
	}

	onMount(() => {
		let disposed = false;
		let unlisten: (() => void) | undefined;
		void inventoryStore.onChange<unknown>((key, value) => {
			if (key === 'items' && Array.isArray(value)) inventoryItems = value as InventoryItem[];
		}).then((stop) => {
			if (disposed) stop(); else unlisten = stop;
		}).catch((cause) => console.error('Could not observe inventory:', cause));
		void waitForInventorySave().catch(() => undefined).then(() => inventoryStore.get<InventoryItem[]>('items'))
			.then((items) => { if (!disposed) inventoryItems = items ?? []; })
			.catch((cause) => console.error('Could not load inventory for listings:', cause));
		return () => { disposed = true; unlisten?.(); };
	});

	async function refresh() {
		if (!marketAccount.session) return;
		loadedFor = marketAccount.session.ingameName;
		loading = true; error = null;
		try {
			const [ordersResponse, items] = await Promise.all([
				invoke<{ data: Listing[] }>('market_my_orders'),
				Object.keys(itemDetails).length ? Promise.resolve(itemDetails) : invoke<Record<string, ListingItem>>('market_item_details'),
			]);
			orders = ordersResponse.data;
			itemDetails = items;
			lastFetchedAt = new Date();
		} catch (cause) { error = String(cause); }
		finally { loading = false; }
	}

	$effect(() => { if (marketAccount.session && loadedFor !== marketAccount.session.ingameName && !loading) void refresh(); });

	async function edit(order: Listing) {
		editing = order; price = order.platinum; quantity = order.quantity; error = null;
		editSellPrices = []; editBuyPrices = []; editMarketMedian = null; editMedianUsesOfferFallback = false;
		editPricesError = null; editPricesLoading = true;
		const requestId = ++editPricesRequestId;
		const slug = detailsFor(order)?.slug ?? order.itemId;
		try {
			const [response, prices] = await Promise.all([
				invoke<{ data: { sell: { platinum: number }[]; buy: { platinum: number }[] } }>(
					'market_top_orders', { slug },
				),
				invoke<Record<string, { median: number; from_current_offers: boolean }>>(
					'get_mastery_tradeable_prices',
				).catch((): Record<string, { median: number; from_current_offers: boolean }> => ({})),
			]);
			if (requestId !== editPricesRequestId) return;
			editSellPrices = response.data.sell.slice(0, 5).map((entry) => entry.platinum);
			editBuyPrices = response.data.buy.slice(0, 5).map((entry) => entry.platinum);
			editMarketMedian = Number.isFinite(prices[slug]?.median) ? prices[slug].median : null;
			editMedianUsesOfferFallback = prices[slug]?.from_current_offers ?? false;
		} catch (cause) {
			if (requestId === editPricesRequestId) editPricesError = String(cause);
		} finally {
			if (requestId === editPricesRequestId) editPricesLoading = false;
		}
	}
	function closeEdit() {
		editPricesRequestId++;
		editPricesLoading = false;
		editing = null;
	}
	async function openEdit(order: Listing) {
		menuOpenFor = null;
		await tick();
		requestAnimationFrame(() => { void edit(order); });
	}
	async function openDelete(order: Listing) {
		menuOpenFor = null;
		await tick();
		requestAnimationFrame(() => { removing = order; error = null; });
	}
	async function setVisibility(order: Listing) {
		if (busy) return;
		menuOpenFor = null; busy = true; error = null;
		try {
			await invoke('market_set_listing_visibility', { id: order.id, visible: !order.visible });
			await refresh();
		} catch (cause) { error = String(cause); }
		finally { busy = false; }
	}
	function setSort(key: string) {
		const column = key as typeof sortColumn;
		if (sortColumn === column) sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		else {
			sortColumn = column;
			sortDirection = column === 'name' ? 'asc' : 'desc';
		}
	}
	async function soldOne(order: Listing, removeInventory: boolean) {
		if (busy || order.type !== 'sell' || (order.perTrade ?? 1) !== 1) return;
		menuOpenFor = null; busy = true; error = null;
		let closed = false;
		try {
			await invoke('market_close_listing_one', { id: order.id });
			closed = true;
			if (removeInventory) await removeOneMarketInventoryItem(detailsFor(order)?.slug, nameFor(order));
			await refresh();
		} catch (cause) {
			if (closed) await refresh();
			error = closed && removeInventory
				? `Listing marked sold. Could not remove one from inventory: ${String(cause)}`
				: String(cause);
		} finally { busy = false; }
	}
	async function save() {
		if (!editing || !valid || busy) return;
		busy = true; error = null;
		try {
			await invoke('market_update_listing', { id: editing.id, platinum: price, quantity });
			closeEdit();
			await refresh();
		} catch (cause) { error = String(cause); }
		finally { busy = false; }
	}
	async function remove() {
		if (!removing || busy) return;
		busy = true; error = null;
		try {
			await invoke('market_delete_listing', { id: removing.id });
			removing = null;
			await refresh();
		} catch (cause) { error = String(cause); }
		finally { busy = false; }
	}
</script>

<div class="mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
	<div class="flex flex-col gap-6">
		<header class="flex flex-wrap items-start justify-between gap-5">
			<div>
				<h1 class="text-3xl font-bold tracking-tight">My Listings</h1>
				<p class="mt-1 text-muted-foreground text-sm">Manage your marketplace listings, track prices, and take action.</p>
			</div>
			<div class="flex flex-wrap items-center gap-2">
				<Button href={marketAccount.session ? marketProfileUrl(marketAccount.session) : undefined} target="_blank" rel="noopener noreferrer" class="inline-flex h-10 items-center gap-2 px-4 text-sm">
					View marketplace <Icon icon="lucide:external-link" class="size-4" />
				</Button>
				<Button variant="primary" onclick={() => (createPickerOpen = true)} class="inline-flex h-10 items-center gap-2 px-4 text-sm font-semibold">
					<Icon icon="lucide:plus" class="size-4" /> Create listing
				</Button>
			</div>
		</header>

		<section aria-label="Listing summary" class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
			<div class="flex items-center gap-4 border border-border-secondary bg-card/50 p-4">
				<div class="flex size-12 shrink-0 items-center justify-center rounded-full bg-sky-500/15 text-sky-400"><Icon icon="lucide:tags" class="size-6" /></div>
				<div><div class="text-2xl font-bold tabular-nums">{orders.length}</div><div class="text-muted-foreground text-sm">Total listings</div></div>
			</div>
			<div class="flex items-center gap-4 border border-border-secondary bg-card/50 p-4">
				<div class="flex size-12 shrink-0 items-center justify-center rounded-full bg-accent/15 text-accent"><Icon icon="lucide:circle-check" class="size-6" /></div>
				<div><div class="text-2xl font-bold tabular-nums">{visibleCount}</div><div class="text-muted-foreground text-sm">Visible</div></div>
			</div>
			<div class="flex items-center gap-4 border border-border-secondary bg-card/50 p-4">
				<div class="flex size-12 shrink-0 items-center justify-center rounded-full bg-amber-500/15 text-amber-400"><Icon icon="lucide:pause" class="size-6" /></div>
				<div><div class="text-2xl font-bold tabular-nums">{hiddenCount}</div><div class="text-muted-foreground text-sm">Hidden</div></div>
			</div>
			<div class="flex items-center gap-4 border border-border-secondary bg-card/50 p-4">
				<div class="flex size-12 shrink-0 items-center justify-center rounded-full bg-violet-500/15 text-violet-400"><Icon icon="lucide:store" class="size-6" /></div>
				<div><div class="text-2xl font-bold tabular-nums">{sellCount}</div><div class="text-muted-foreground text-sm">Sell listings</div></div>
			</div>
		</section>

		{#if loading && orders.length === 0}
			<div class="border border-border-secondary bg-card/50 p-10 text-center text-muted-foreground text-sm">Loading listings...</div>
		{:else if error && !editing && !removing && orders.length === 0}
			<div class="border border-border-secondary bg-card/50 p-10 text-center">
				<p role="alert" class="mb-3 text-danger text-sm">{error}</p>
				<Button onclick={refresh}>Try again</Button>
			</div>
		{:else if orders.length === 0}
			<div class="flex flex-col items-center border border-border-secondary bg-card/50 px-6 py-12 text-center">
				<div class="mb-4 flex size-12 items-center justify-center rounded-full border border-border-secondary bg-surface text-muted-foreground"><Icon icon="material-symbols:format-list-bulleted-rounded" class="size-6" /></div>
				<h2 class="font-semibold text-base">No listings yet</h2>
				<p class="mt-1 text-muted-foreground text-sm">Create a sell listing for a market item.</p>
				<Button variant="primary" class="mt-5 inline-flex items-center gap-1.5" onclick={() => (createPickerOpen = true)}><Icon icon="lucide:plus" class="size-4" /> Create listing</Button>
			</div>
		{:else}
			<div class="flex flex-col gap-4">
				{#if error && !editing && !removing}<p role="alert" class="text-danger text-sm">{error}</p>{/if}
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
						<Select type="single" bind:value={typeFilter} items={typeFilterItems} placeholder="Listing type" triggerProps={{ id: 'listings-type-filter', class: 'h-11 max-w-none border-border-secondary bg-background/70' }} />
					</label>
					<label for="listings-status-filter" class="block min-w-0">
						<span class="mb-1.5 block font-semibold text-muted-foreground text-xs">Status</span>
						<Select type="single" bind:value={statusFilter} items={statusFilterItems} placeholder="Status" triggerProps={{ id: 'listings-status-filter', class: 'h-11 max-w-none border-border-secondary bg-background/70' }} />
					</label>
				</div>
				{#if sortedOrders.length === 0}
					<div class="border border-border-secondary bg-card/50 p-8 text-center text-muted-foreground text-sm">No listings match this search.</div>
				{:else}
				<div class="overflow-hidden border border-border-secondary [&>div]:border-0">
				<Table {columns} {sortColumn} {sortDirection} onSort={setSort} minWidth="900px">
					{#each sortedOrders as order (order.id)}
						<tr class="border-t border-border-secondary transition-colors hover:bg-surface/70">
							<td class="px-3 py-4 min-w-64">
								<div class="flex items-center gap-3">
									<div class="flex size-12 shrink-0 items-center justify-center overflow-hidden border border-border-secondary bg-background/70">
										{#if detailsFor(order)?.icon}
											<img src={`https://warframe.market/static/assets/${detailsFor(order)?.icon}`} alt="" loading="lazy" class="size-11 object-contain" />
										{:else}
											<Icon icon="lucide:package" class="size-5 text-muted-foreground" />
										{/if}
									</div>
									<div class="min-w-0">
										<div class="font-semibold text-foreground">{nameFor(order)}</div>
										{#if order.rank != null || order.subtype}<div class="mt-0.5 text-muted-foreground text-xs">{order.rank != null ? `Rank ${order.rank}` : ''}{order.rank != null && order.subtype ? ' · ' : ''}{order.subtype ?? ''}</div>{/if}
									</div>
								</div>
							</td>
							<td class="px-3 py-4"><span class={`inline-flex rounded-full border px-3 py-1 text-xs font-semibold ${order.type === 'sell' ? 'border-sky-500/30 bg-sky-500/15 text-sky-300' : 'border-violet-500/30 bg-violet-500/15 text-violet-300'}`}>{order.type === 'sell' ? 'Sell' : 'Buy'}</span></td>
							<td class="px-3 py-4 text-right font-semibold tabular-nums"><span class="inline-flex items-center justify-end gap-1.5">{order.platinum}<img src="/icons/platinum.png" class="size-3.5" alt="platinum" /></span></td>
							<td class="px-3 py-4 text-right font-semibold tabular-nums">{order.quantity}</td>
							<td class="px-3 py-4"><span class={`inline-flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-semibold ${order.visible ? 'border-accent/30 bg-accent/10 text-accent' : 'border-amber-500/30 bg-amber-500/10 text-amber-400'}`}><span class={`size-1.5 rounded-full ${order.visible ? 'bg-accent' : 'bg-amber-400'}`}></span>{order.visible ? 'Visible' : 'Hidden'}</span></td>
							<td class="px-3 py-4 text-right">
								<div class="flex items-center justify-end gap-2">
									<Button disabled={busy} onclick={() => openEdit(order)} class="inline-flex h-9 items-center gap-1.5 px-3 text-xs"><Icon icon="lucide:pencil" class="size-3.5" /> Edit</Button>
								<ActionPopover bind:open={() => menuOpenFor === order.id, (value) => { menuOpenFor = value ? order.id : null; }} triggerAriaLabel={`Actions for ${nameFor(order)}`} triggerClass="inline-flex items-center justify-center hover:bg-elevated border border-border-secondary size-9" contentClass="w-60">
									{#snippet trigger()}<Icon icon="lucide:ellipsis" class="size-4" />{/snippet}
									<button type="button" onclick={() => { menuOpenFor = null; onOpenMarket(slugFor(order)); }} class="flex items-center gap-2 hover:bg-elevated px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:store" class="size-4" /> View market</button>
									<button type="button" disabled={busy} onclick={() => setVisibility(order)} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon={order.visible ? 'lucide:eye-off' : 'lucide:eye'} class="size-4" /> {order.visible ? 'Hide listing' : 'Unhide listing'}</button>
									{#if order.type === 'sell'}
										<button type="button" disabled={busy || (order.perTrade ?? 1) !== 1} title={(order.perTrade ?? 1) !== 1 ? 'This listing must be sold in larger trade units' : undefined} onclick={() => soldOne(order, false)} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:check" class="size-4" /> Sold 1</button>
										<button type="button" disabled={busy || (order.perTrade ?? 1) !== 1 || ownedFor(order) < 1} title={ownedFor(order) < 1 ? 'No matching item in inventory' : undefined} onclick={() => soldOne(order, true)} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:package-minus" class="size-4" /> Sold 1 and remove 1 from inventory</button>
									{/if}
									<button type="button" disabled={busy} onclick={() => openDelete(order)} class="flex items-center gap-2 text-danger hover:bg-danger/15 disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:trash-2" class="size-4" /> Delete listing</button>
								</ActionPopover>
								</div>
							</td>
						</tr>
					{/each}
				</Table>
				</div>
				{/if}
				<div class="flex flex-wrap items-center justify-between gap-3 border border-border-secondary bg-card/30 px-4 py-3">
					<p class="text-muted-foreground text-sm">Showing {sortedOrders.length} of {orders.length} listings</p>
					<div class="flex flex-wrap items-center gap-3">
						{#if lastFetchedAt}<time datetime={lastFetchedAt.toISOString()} class="text-muted-foreground text-xs">Last refreshed: {fetchedAtFormatter.format(lastFetchedAt)}</time>{/if}
						<Button disabled={loading} onclick={refresh} class="inline-flex items-center gap-2 text-xs"><Icon icon="lucide:refresh-cw" class={`size-4 ${loading ? 'animate-spin' : ''}`} /> Refresh</Button>
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>

{#snippet createTitle()}Create sell listing{/snippet}
{#snippet createDescription()}Search the market item list to create a listing. You can list items you do not own.{/snippet}
{#snippet createClose()}<Button>Cancel</Button>{/snippet}
{#snippet createActions()}<Button variant="primary" disabled={!selectedCreateItem} onclick={selectCreateItem}>Continue</Button>{/snippet}
<Dialog bind:open={createPickerOpen} title={createTitle} description={createDescription} dialogClose={createClose} dialogActions={createActions} contentProps={{ class: 'h-auto' }}>
	<div class="px-6">
		<label for="listing-create-item" class="block mb-1.5 font-semibold text-muted-foreground text-xs">Item</label>
		<Combobox type="single" items={catalogItems} bind:value={selectedCreateSlug} inputValue={selectedCreateItem?.label ?? ''} disabled={catalogLoading || !!catalogError} inputProps={{ id: 'listing-create-item', placeholder: catalogLoading ? 'Loading items...' : 'Search for an item...' }} />
		{#if catalogError}<p role="alert" class="mt-2 text-danger text-sm">Could not load items. <button type="button" class="underline cursor-pointer" onclick={loadCatalog}>Retry</button></p>{/if}
	</div>
</Dialog>
<CreateListing bind:item={createItem} mastered={createItem ? isMarketItemMastered(masteredIndex, createItem.slug, createItem.name) : false} onCreated={refresh} />

{#snippet editTitle()}{editing?.type === 'sell' ? 'Edit sell listing' : 'Edit buy order'}{/snippet}
{#snippet editDescription()}{#if editing}<ListingItemInfo name={nameFor(editing)} mastered={editing.type === 'sell' && masteredFor(editing)} ownedCount={editing.type === 'sell' ? ownedFor(editing) : 0} />{/if}{/snippet}
{#snippet editClose()}<Button>Cancel</Button>{/snippet}
{#snippet editActions()}<Button variant="primary" disabled={!valid || busy} onclick={save}>Save changes</Button>{/snippet}

<Dialog bind:open={() => editing !== null, (value) => { if (!value) closeEdit(); }} title={editTitle} description={editDescription} dialogClose={editClose} dialogActions={editActions} contentProps={{ class: 'h-auto max-h-[calc(100vh-2rem)]' }}>
	<div class="grid grid-cols-2 gap-3 px-6">
		{#if editPricesLoading}
			<p class="col-span-2 text-muted-foreground text-sm">Loading current orders...</p>
		{:else if editPricesError}
			<p role="alert" class="col-span-2 text-danger text-sm">Could not load current orders: {editPricesError}</p>
		{:else}
			<div class="col-span-2 grid grid-cols-2 gap-4">
				<div class="bg-card/50 p-3 border border-border-secondary">
					<h3 class="mb-2 font-semibold text-sm">Top sell orders</h3>
					{#each editSellPrices as value}<div class="py-0.5 tabular-nums text-sm">{value} platinum</div>{:else}<p class="text-muted-foreground text-sm">No sell orders</p>{/each}
					{#if editing?.type === 'sell'}
						<div class="mt-2 pt-2 border-t border-border-secondary font-semibold text-sm" title={editMedianUsesOfferFallback ? 'Current offer median; no recent trade median' : 'Recent trade median'}>
							Market median: {editMarketMedian === null ? '—' : `${editMedianUsesOfferFallback ? '~' : ''}${editMarketMedian} platinum`}
						</div>
					{/if}
				</div>
				<div class="bg-card/50 p-3 border border-border-secondary">
					<h3 class="mb-2 font-semibold text-sm">Top buy orders</h3>
					{#each editBuyPrices as value}<div class="py-0.5 tabular-nums text-sm">{value} platinum</div>{:else}<p class="text-muted-foreground text-sm">No buy orders</p>{/each}
				</div>
			</div>
		{/if}
		<label class="flex flex-col gap-1 text-sm">Price (platinum)<input type="number" min="1" max="900000" step="1" bind:value={price} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" /></label>
		<label class="flex flex-col gap-1 text-sm">Quantity<input type="number" min="1" max="9999" step="1" bind:value={quantity} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" /></label>
		{#if editing?.type === 'sell'}<div class="col-span-2"><ListingQuantityWarning {quantity} ownedCount={ownedFor(editing)} /></div>{/if}
		{#if error}<p role="alert" class="col-span-2 text-danger text-sm">{error}</p>{/if}
	</div>
</Dialog>

{#snippet removeTitle()}Delete listing?{/snippet}
{#snippet removeDescription()}This deletes the order from warframe.market.{/snippet}
{#snippet removeClose()}<Button>Cancel</Button>{/snippet}
{#snippet removeActions()}<Button disabled={busy} onclick={remove} class="border-danger bg-danger text-danger-foreground hover:bg-danger/80">Delete listing</Button>{/snippet}
<Dialog bind:open={() => removing !== null, (value) => { if (!value) removing = null; }} title={removeTitle} description={removeDescription} dialogClose={removeClose} dialogActions={removeActions} contentProps={{ class: 'h-auto' }}>
	<div class="px-6 text-sm">{removing ? nameFor(removing) : ''}{#if error}<p role="alert" class="mt-2 text-danger">{error}</p>{/if}</div>
</Dialog>
