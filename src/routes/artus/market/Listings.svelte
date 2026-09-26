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
	import { marketAccount, marketProfileUrl } from '$lib/market-account.svelte';
	import { removeOneMarketInventoryItem, waitForInventorySave, type InventoryItem } from '$lib/inventory';
	import { mastery } from '$lib/mastery.svelte';
	import { isMarketItemMastered, masteredMarketItems, ownedMarketCount } from '$lib/listing-context';
	import { DictionarySchema } from '$lib/schemas';
	import CreateListing from '../inventory/CreateListing.svelte';

	type Listing = { id: string; itemId: string; type: 'buy' | 'sell'; platinum: number; quantity: number; visible: boolean; perTrade?: number; rank?: number; subtype?: string };
	type ListingItem = { name: string; slug: string };
	const inventoryStore = new LazyStore('inventory.json');
	let orders = $state<Listing[]>([]);
	let itemDetails = $state<Record<string, ListingItem>>({});
	let inventoryItems = $state<InventoryItem[]>([]);
	let loading = $state(false);
	let busy = $state(false);
	let error = $state<string | null>(null);
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
	let removing = $state<Listing | null>(null);
	let price = $state(1);
	let quantity = $state(1);
	const columns: TableColumn[] = [
		{ key: 'item', label: 'Item', class: 'min-w-48' },
		{ key: 'type', label: 'Type', class: 'w-24' },
		{ key: 'price', label: 'Price', align: 'right', class: 'w-28' },
		{ key: 'quantity', label: 'Quantity', align: 'right', class: 'w-24' },
		{ key: 'visibility', label: 'Visibility', class: 'w-24' },
		{ key: 'actions', label: 'Actions', align: 'right', class: 'w-20' },
	];
	const sellCount = $derived(orders.filter((order) => order.type === 'sell').length);
	const buyCount = $derived(orders.length - sellCount);
	const valid = $derived(Number.isSafeInteger(price) && price >= 1 && price <= 900000 && Number.isSafeInteger(quantity) && quantity >= 1 && quantity <= 9999);
	const masteredIndex = $derived(masteredMarketItems(mastery.items, mastery.checked));
	const detailsFor = (order: Listing) => itemDetails[order.itemId];
	const nameFor = (order: Listing) => detailsFor(order)?.name ?? order.itemId;
	const ownedFor = (order: Listing) => ownedMarketCount(inventoryItems, detailsFor(order)?.slug, nameFor(order));
	const masteredFor = (order: Listing) => isMarketItemMastered(masteredIndex, detailsFor(order)?.slug, nameFor(order));
	const selectedCreateItem = $derived(catalogItems.find((item) => item.value === selectedCreateSlug));
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

	function edit(order: Listing) { editing = order; price = order.platinum; quantity = order.quantity; error = null; }
	async function openEdit(order: Listing) {
		menuOpenFor = null;
		await tick();
		requestAnimationFrame(() => edit(order));
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
			editing = null;
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

<div class="flex flex-col items-center gap-4 mx-auto p-8 w-full">
	<div class="flex flex-col gap-6 w-full max-w-5xl">
		<div class="bg-card/50 p-4 border border-border-secondary">
			<div class="flex flex-wrap justify-between items-start gap-4">
				<div>
					<div class="mb-3 font-semibold text-muted-foreground text-xs uppercase tracking-wider">My listings</div>
					<div class="flex flex-wrap items-baseline gap-x-8 gap-y-2">
						<div class="flex items-baseline gap-2"><span class="font-semibold tabular-nums text-xl">{orders.length}</span><span class="text-muted-foreground text-sm">total</span></div>
						<div class="flex items-baseline gap-2"><span class="font-semibold tabular-nums text-lg">{sellCount}</span><span class="text-muted-foreground text-sm">sell</span></div>
						<div class="flex items-baseline gap-2"><span class="font-semibold tabular-nums text-lg">{buyCount}</span><span class="text-muted-foreground text-sm">buy</span></div>
					</div>
					<p class="mt-2 text-muted-foreground text-xs">Current orders on warframe.market</p>
				</div>
				<div class="flex flex-wrap items-center gap-2">
					<Button variant="primary" onclick={() => (createPickerOpen = true)} class="inline-flex items-center gap-1.5 text-xs"><Icon icon="lucide:plus" class="size-3.5" /> Create listing</Button>
					<Button href={marketAccount.session ? marketProfileUrl(marketAccount.session) : undefined} target="_blank" rel="noopener noreferrer" class="inline-flex items-center gap-1.5 text-xs"><Icon icon="lucide:external-link" class="size-3.5" /> View on warframe.market</Button>
					<Button disabled={loading} onclick={refresh} class="inline-flex items-center gap-1.5 text-xs"><Icon icon="lucide:refresh-cw" class="size-3.5" /> Refresh</Button>
					{#if lastFetchedAt}<time datetime={lastFetchedAt.toISOString()} class="text-muted-foreground text-xs">Last fetched at {fetchedAtFormatter.format(lastFetchedAt)}</time>{/if}
				</div>
			</div>
		</div>

		{#if loading && orders.length === 0}
			<div class="bg-card/50 p-10 border border-border-secondary text-muted-foreground text-sm text-center">Loading listings...</div>
		{:else if error && !editing && !removing && orders.length === 0}
			<div class="bg-card/50 p-10 border border-border-secondary text-center">
				<p role="alert" class="mb-3 text-danger text-sm">{error}</p>
				<Button onclick={refresh}>Try again</Button>
			</div>
		{:else if orders.length === 0}
			<div class="flex flex-col items-center bg-card/50 px-6 py-12 border border-border-secondary text-center">
				<div class="flex items-center justify-center bg-surface mb-4 border border-border-secondary size-12 text-muted-foreground"><Icon icon="material-symbols:format-list-bulleted-rounded" class="size-6" /></div>
				<h2 class="font-semibold text-base">No listings yet</h2>
				<p class="mt-1 text-muted-foreground text-sm">Create a sell listing for a market item.</p>
				<Button variant="primary" class="mt-5 inline-flex items-center gap-1.5" onclick={() => (createPickerOpen = true)}><Icon icon="lucide:plus" class="size-4" /> Create listing</Button>
			</div>
		{:else}
			<div class="flex flex-col gap-3">
				{#if error && !editing && !removing}<p role="alert" class="text-danger text-sm">{error}</p>{/if}
				<Table {columns} minWidth="760px">
					{#each orders as order (order.id)}
						<tr class="border-t border-border-secondary transition-colors hover:bg-surface/70">
							<td class="px-3 py-3.5 font-semibold min-w-48">
								<div>{nameFor(order)}</div>
								{#if order.rank != null || order.subtype}<div class="mt-0.5 text-muted-foreground text-xs font-normal">{order.rank != null ? `Rank ${order.rank}` : ''}{order.rank != null && order.subtype ? ' · ' : ''}{order.subtype ?? ''}</div>{/if}
							</td>
							<td class="px-3 py-3.5 text-sm capitalize">{order.type}</td>
							<td class="px-3 py-3.5 text-right tabular-nums"><span class="inline-flex items-center justify-end gap-1">{order.platinum}<img src="/icons/platinum.png" class="size-3.5" alt="platinum" /></span></td>
							<td class="px-3 py-3.5 text-right tabular-nums">{order.quantity}</td>
							<td class="px-3 py-3.5"><span class={order.visible ? 'text-accent' : 'text-muted-foreground'}>{order.visible ? 'Visible' : 'Hidden'}</span></td>
							<td class="px-3 py-3.5 text-right">
								<ActionPopover bind:open={() => menuOpenFor === order.id, (value) => { menuOpenFor = value ? order.id : null; }} triggerAriaLabel={`Actions for ${nameFor(order)}`} triggerClass="inline-flex items-center justify-center hover:bg-elevated border border-border-secondary size-7" contentClass="w-60">
									{#snippet trigger()}<Icon icon="lucide:ellipsis" class="size-4" />{/snippet}
									<button type="button" disabled={busy} onclick={() => openEdit(order)} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:pencil" class="size-4" /> Edit listing</button>
									<button type="button" disabled={busy} onclick={() => setVisibility(order)} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon={order.visible ? 'lucide:eye-off' : 'lucide:eye'} class="size-4" /> {order.visible ? 'Hide listing' : 'Unhide listing'}</button>
									{#if order.type === 'sell'}
										<button type="button" disabled={busy || (order.perTrade ?? 1) !== 1} title={(order.perTrade ?? 1) !== 1 ? 'This listing must be sold in larger trade units' : undefined} onclick={() => soldOne(order, false)} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:check" class="size-4" /> Sold 1</button>
										<button type="button" disabled={busy || (order.perTrade ?? 1) !== 1 || ownedFor(order) < 1} title={ownedFor(order) < 1 ? 'No matching item in inventory' : undefined} onclick={() => soldOne(order, true)} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:package-minus" class="size-4" /> Sold 1 and remove 1 from inventory</button>
									{/if}
									<button type="button" disabled={busy} onclick={() => openDelete(order)} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-40 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:trash-2" class="size-4" /> Delete listing</button>
								</ActionPopover>
							</td>
						</tr>
					{/each}
				</Table>
				<p class="text-muted-foreground text-sm">Showing {orders.length} listings</p>
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
<Dialog bind:open={() => editing !== null, (value) => { if (!value) editing = null; }} title={editTitle} description={editDescription} dialogClose={editClose} dialogActions={editActions} contentProps={{ class: 'h-auto' }}>
	<div class="grid grid-cols-2 gap-3 px-6">
		<label class="flex flex-col gap-1 text-sm">Price (platinum)<input type="number" min="1" max="900000" step="1" bind:value={price} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" /></label>
		<label class="flex flex-col gap-1 text-sm">Quantity<input type="number" min="1" max="9999" step="1" bind:value={quantity} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" /></label>
		{#if editing?.type === 'sell'}<div class="col-span-2"><ListingQuantityWarning {quantity} ownedCount={ownedFor(editing)} /></div>{/if}
		{#if error}<p role="alert" class="col-span-2 text-danger text-sm">{error}</p>{/if}
	</div>
</Dialog>

{#snippet removeTitle()}Delete listing?{/snippet}
{#snippet removeDescription()}This deletes the order from warframe.market.{/snippet}
{#snippet removeClose()}<Button>Cancel</Button>{/snippet}
{#snippet removeActions()}<Button variant="primary" disabled={busy} onclick={remove}>Delete listing</Button>{/snippet}
<Dialog bind:open={() => removing !== null, (value) => { if (!value) removing = null; }} title={removeTitle} description={removeDescription} dialogClose={removeClose} dialogActions={removeActions} contentProps={{ class: 'h-auto' }}>
	<div class="px-6 text-sm">{removing ? nameFor(removing) : ''}{#if error}<p role="alert" class="mt-2 text-danger">{error}</p>{/if}</div>
</Dialog>
