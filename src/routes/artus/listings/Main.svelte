<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount, tick } from 'svelte';
	import { LazyStore } from '@tauri-apps/plugin-store';
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import { marketAccount, marketProfileUrl } from '$lib/market-account.svelte';
	import {
		removeOneMarketInventoryItem,
		waitForInventorySave,
		type InventoryItem,
	} from '$lib/inventory';
	import { mastery } from '$lib/mastery.svelte';
	import { timeAgo } from '$lib/date';
	import { fetchMarketListings } from '$lib/market-listings';
	import { isMarketItemMastered, masteredMarketItems } from '$lib/listing-context';
	import CreateListing from '../inventory/CreateListing.svelte';
	import CreateListingPicker from './CreateListingPicker.svelte';
	import ListingFilters from './ListingFilters.svelte';
	import ListingSummary from './ListingSummary.svelte';
	import ListingTable from './ListingTable.svelte';
	import { listingName, listingOwned, listingSlug } from './listing-utils';
	import type { Listing, ListingChange, ListingItem } from './types';

	let { onOpenMarket = () => {} }: { onOpenMarket?: (slug: string) => void } = $props();
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
	let sortColumn = $state<'name' | 'price' | 'quantity'>('name');
	let sortDirection = $state<'asc' | 'desc'>('asc');
	let loadedFor = $state<string | null>(null);
	let lastFetchedAt = $state<Date | null>(null);
	let now = $state(Date.now());
	let fetchedAgo = $derived(lastFetchedAt === null ? '' : timeAgo(lastFetchedAt.getTime(), now));
	let createPickerOpen = $state(false);
	let activeItem = $state<InventoryItem | null>(null);
	let editing = $state<Listing | null>(null);
	let removing = $state<Listing | null>(null);
	const masteredIndex = $derived(masteredMarketItems(mastery.items, mastery.checked));
	const nameFor = (order: Listing) => listingName(order, itemDetails);
	const slugFor = (order: Listing) => listingSlug(order, itemDetails);
	const ownedFor = (order: Listing) => listingOwned(order, itemDetails, inventoryItems);
	const filteredOrders = $derived.by(() => {
		const query = search.trim().toLocaleLowerCase();
		return orders.filter((order) => {
			if (typeFilter !== 'all' && order.type !== typeFilter) return false;
			if (statusFilter !== 'all' && order.visible !== (statusFilter === 'visible')) return false;
			if (!query) return true;
			return `${nameFor(order)} ${order.itemId} ${order.type} ${order.subtype ?? ''}`
				.toLocaleLowerCase()
				.includes(query);
		});
	});
	const sortedOrders = $derived.by(() => {
		const direction = sortDirection === 'asc' ? 1 : -1;
		return [...filteredOrders].sort((a, b) => {
			let comparison: number;
			switch (sortColumn) {
				case 'name':
					comparison = nameFor(a).localeCompare(nameFor(b), undefined, { sensitivity: 'base' });
					break;
				case 'price':
					comparison = a.platinum - b.platinum;
					break;
				case 'quantity':
					comparison = a.quantity - b.quantity;
					break;
			}
			const byName = nameFor(a).localeCompare(nameFor(b), undefined, { sensitivity: 'base' });
			return direction * (comparison || byName || a.id.localeCompare(b.id));
		});
	});
	onMount(() => {
		let disposed = false;
		let unlisten: (() => void) | undefined;
		const timer = setInterval(() => {
			now = Date.now();
		}, 1000);
		void inventoryStore
			.onChange<unknown>((key, value) => {
				if (key === 'items' && Array.isArray(value)) inventoryItems = value as InventoryItem[];
			})
			.then((stop) => {
				if (disposed) stop();
				else unlisten = stop;
			})
			.catch((cause) => console.error('Could not observe inventory:', cause));
		void waitForInventorySave()
			.catch(() => undefined)
			.then(() => inventoryStore.get<InventoryItem[]>('items'))
			.then((items) => {
				if (!disposed) inventoryItems = items ?? [];
			})
			.catch((cause) => console.error('Could not load inventory for listings:', cause));
		return () => {
			disposed = true;
			clearInterval(timer);
			unlisten?.();
		};
	});

	async function refresh() {
		if (!marketAccount.session || loading || busy) return;
		loadedFor = marketAccount.session.ingameName;
		loading = true;
		error = null;
		try {
			const [fetchedOrders, items] = await Promise.all([
				fetchMarketListings(),
				Object.keys(itemDetails).length
					? Promise.resolve(itemDetails)
					: invoke<Record<string, ListingItem>>('market_item_details'),
			]);
			orders = fetchedOrders;
			itemDetails = items;
			lastFetchedAt = new Date();
		} catch (cause) {
			error = String(cause);
		} finally {
			loading = false;
		}
	}

	function applyListingChange(change: ListingChange) {
		switch (change.kind) {
			case 'created':
				if (change.listing) orders = [...orders, change.listing];
				else error = 'Listing created. Refresh to show it in the list.';
				break;
			case 'updated':
				orders = orders.map((order) => order.id === change.id
					? { ...order, platinum: change.platinum, quantity: change.quantity } : order);
				break;
			case 'visibility':
				orders = orders.map((order) => order.id === change.id
					? { ...order, visible: change.visible } : order);
				break;
			case 'deleted':
				orders = orders.filter((order) => order.id !== change.id);
				break;
		}
	}

	$effect(() => {
		if (marketAccount.session && loadedFor !== marketAccount.session.ingameName && !loading) {
			void refresh();
		}
	});

	async function openEdit(order: Listing) {
		await tick();
		requestAnimationFrame(() => {
			editing = order;
			activeItem = {
				name: nameFor(order),
				slug: slugFor(order),
				quantity: order.type === 'sell' ? ownedFor(order) : 0,
			};
		});
	}
	async function openDelete(order: Listing) {
		await tick();
		requestAnimationFrame(() => {
			removing = order;
			error = null;
		});
	}
	async function setVisibility(order: Listing) {
		if (busy) return;
		busy = true;
		error = null;
		try {
			await invoke('market_set_listing_visibility', { id: order.id, visible: !order.visible });
			applyListingChange({ kind: 'visibility', id: order.id, visible: !order.visible });
		} catch (cause) {
			error = String(cause);
		} finally {
			busy = false;
		}
	}
	async function setAllVisibility(visible: boolean) {
		if (busy) return;
		const targets = orders.filter((order) => order.visible !== visible);
		if (!targets.length) return;
		busy = true;
		error = null;
		let updateError: string | null = null;
		try {
			for (const order of targets) {
				await invoke('market_set_listing_visibility', { id: order.id, visible });
				applyListingChange({ kind: 'visibility', id: order.id, visible });
			}
		} catch (cause) {
			updateError = String(cause);
		} finally {
			if (updateError) error = updateError;
			busy = false;
		}
	}
	function setSort(key: string) {
		const column = key as typeof sortColumn;
		if (sortColumn === column) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortColumn = column;
			sortDirection = column === 'name' ? 'asc' : 'desc';
		}
	}
	async function soldOne(order: Listing) {
		if (busy || order.type !== 'sell' || (order.perTrade ?? 1) !== 1) return;
		busy = true;
		error = null;
		let closed = false;
		try {
			await invoke('market_close_listing_one', { id: order.id });
			closed = true;
			if (ownedFor(order) > 0) {
				await removeOneMarketInventoryItem(itemDetails[order.itemId]?.slug, nameFor(order));
			}
			orders = orders.flatMap((listing) => listing.id !== order.id ? [listing]
				: listing.quantity > 1 ? [{ ...listing, quantity: listing.quantity - 1 }] : []);
		} catch (cause) {
			if (closed) orders = orders.flatMap((listing) => listing.id !== order.id ? [listing]
				: listing.quantity > 1 ? [{ ...listing, quantity: listing.quantity - 1 }] : []);
			error = closed
				? `Listing marked sold. Could not remove one from inventory: ${String(cause)}`
				: String(cause);
		} finally {
			busy = false;
		}
	}
	async function remove() {
		if (!removing || busy) return;
		busy = true;
		error = null;
		try {
			await invoke('market_delete_listing', { id: removing.id });
			applyListingChange({ kind: 'deleted', id: removing.id });
			removing = null;
		} catch (cause) {
			error = String(cause);
		} finally {
			busy = false;
		}
	}
</script>

<div class="mx-auto py-6 w-full max-w-5xl">
	<div class="flex flex-col gap-4">
		<header class="flex flex-wrap justify-between items-center gap-4 w-full">
			<ListingSummary {orders} />
			<div class="flex flex-wrap items-center gap-2">
				{#if lastFetchedAt}<time
						datetime={lastFetchedAt.toISOString()}
						class="text-muted-foreground text-xs"
					>
						refreshed {fetchedAgo}
					</time>{/if}
				<Button disabled={loading || busy} onclick={refresh} class="h-full" size="icon">
					<Icon icon="lucide:refresh-cw" class={`size-4 ${loading ? 'animate-spin' : ''}`} />
				</Button>
				<Button
					href={marketAccount.session ? marketProfileUrl(marketAccount.session) : undefined}
					target="_blank"
					rel="noopener noreferrer"
					class="text-sm"
				>
					View profile <Icon icon="material-symbols:arrow-outward-rounded" class="inline size-4" />
				</Button>
				<Button variant="primary" onclick={() => (createPickerOpen = true)} class="text-sm">
					<Icon icon="lucide:plus" class="inline size-4" /> Create listing
				</Button>
			</div>
		</header>
		<div class="bg-surface my-1 w-full h-px"></div>
		{#if loading && orders.length === 0}
			<div role="status" aria-label="Loading listings" class="flex flex-col gap-4">
				<div class="flex gap-3">
					<Skeleton class="flex-1 h-11" />
					<Skeleton class="w-32 h-11" />
					<Skeleton class="w-32 h-11" />
				</div>
				<div class="bg-card/50 border border-border-secondary divide-y divide-border-secondary">
					{#each Array(6) as _}
						<div class="flex items-center gap-4 px-3 py-3.5 h-14">
							<Skeleton class="flex-1 max-w-64 h-4" />
							<Skeleton class="w-14 h-6" />
							<Skeleton class="w-12 h-4" />
							<Skeleton class="w-12 h-4" />
							<Skeleton class="w-16 h-4" />
						</div>
					{/each}
				</div>
			</div>
		{:else if error && !editing && !removing && orders.length === 0}
			<div class="bg-card/50 p-10 border border-border-secondary text-center">
				<p role="alert" class="mb-3 text-danger text-sm">{error}</p>
				<Button onclick={refresh}>Try again</Button>
			</div>
		{:else if orders.length === 0}
			<div
				class="flex flex-col items-center bg-card/50 px-6 py-12 border border-border-secondary text-center"
			>
				<div
					class="flex justify-center items-center bg-surface mb-4 border border-border-secondary rounded-full size-12 text-muted-foreground"
				>
					<Icon icon="material-symbols:format-list-bulleted-rounded" class="size-6" />
				</div>
				<h2 class="font-semibold text-base">No listings yet</h2>
				<p class="mt-1 text-muted-foreground text-sm">Create a sell listing for a market item.</p>
				<Button
					variant="primary"
					class="inline-flex items-center gap-1.5 mt-5"
					onclick={() => (createPickerOpen = true)}
				>
					<Icon icon="lucide:plus" class="size-4" /> Create listing
				</Button>
			</div>
		{:else}
			<div class="flex flex-col gap-4">
				{#if error && !editing && !removing}<p role="alert" class="text-danger text-sm">
						{error}
					</p>{/if}
				<ListingFilters bind:search bind:typeFilter bind:statusFilter />
				{#if sortedOrders.length === 0}
					<div
						class="bg-card/50 p-8 border border-border-secondary text-muted-foreground text-sm text-center"
					>
						No listings match this search.
					</div>
				{:else}
					<ListingTable
						orders={sortedOrders}
						{itemDetails}
						{inventoryItems}
						{busy}
						{sortColumn}
						{sortDirection}
						onSort={setSort}
						onEdit={openEdit}
						onVisibility={setVisibility}
						onSoldOne={soldOne}
						onDelete={openDelete}
						{onOpenMarket}
					/>
				{/if}
				<div class="flex flex-wrap justify-between items-center gap-3 bg-card/30">
					<p class="text-muted-foreground text-sm">
						Showing {sortedOrders.length} of {orders.length} listings
					</p>
					<div class="flex items-center gap-2">
						<Button
							disabled={busy || !orders.some((order) => order.visible)}
							onclick={() => setAllVisibility(false)}
						>
							Hide all
						</Button>
						<Button
							disabled={busy || !orders.some((order) => !order.visible)}
							onclick={() => setAllVisibility(true)}
						>
							Show all
						</Button>
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>

<CreateListingPicker
	bind:open={createPickerOpen}
	{inventoryItems}
	onSelect={(item) => {
		editing = null;
		activeItem = item;
	}}
/>
<CreateListing
	bind:item={
		() => activeItem,
		(value) => {
			activeItem = value;
			if (value === null) editing = null;
		}
	}
	{editing}
	mastered={activeItem && editing?.type !== 'buy'
		? isMarketItemMastered(masteredIndex, activeItem.slug, activeItem.name)
		: false}
	onSaved={applyListingChange}
/>

{#snippet removeTitle()}Delete listing?{/snippet}
{#snippet removeDescription()}This deletes the order from warframe.market.{/snippet}
{#snippet removeClose()}<Button>Cancel</Button>{/snippet}
{#snippet removeActions()}<Button
		disabled={busy}
		onclick={remove}
		class="bg-danger hover:bg-danger/80 border-danger text-danger-foreground"
	>
		Delete listing
	</Button>{/snippet}
<Dialog
	bind:open={
		() => removing !== null,
		(value) => {
			if (!value) removing = null;
		}
	}
	title={removeTitle}
	description={removeDescription}
	dialogClose={removeClose}
	dialogActions={removeActions}
	contentProps={{ class: 'h-auto' }}
>
	<div class="px-6 text-sm">
		{removing ? nameFor(removing) : ''}{#if error}<p role="alert" class="mt-2 text-danger">
				{error}
			</p>{/if}
	</div>
</Dialog>
