<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import { marketAccount } from '$lib/market-account.svelte';

	type Listing = { id: string; itemId: string; type: 'buy' | 'sell'; platinum: number; quantity: number; visible: boolean; rank?: number; subtype?: string };
	let orders = $state<Listing[]>([]);
	let names = $state<Record<string, string>>({});
	let loading = $state(false);
	let busy = $state(false);
	let error = $state<string | null>(null);
	let loadedFor = $state<string | null>(null);
	let editing = $state<Listing | null>(null);
	let removing = $state<Listing | null>(null);
	let price = $state(1);
	let quantity = $state(1);
	const valid = $derived(Number.isSafeInteger(price) && price >= 1 && price <= 900000 && Number.isSafeInteger(quantity) && quantity >= 1 && quantity <= 9999);

	async function refresh() {
		if (!marketAccount.session) return;
		loadedFor = marketAccount.session.ingameName;
		loading = true; error = null;
		try {
			const [ordersResponse, itemNames] = await Promise.all([
				invoke<{ data: Listing[] }>('market_my_orders'),
				invoke<Record<string, string>>('market_item_names'),
			]);
			orders = ordersResponse.data;
			names = itemNames;
		} catch (cause) { error = String(cause); }
		finally { loading = false; }
	}

	$effect(() => { if (marketAccount.session && loadedFor !== marketAccount.session.ingameName && !loading) void refresh(); });

	function edit(order: Listing) { editing = order; price = order.platinum; quantity = order.quantity; error = null; }
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

<section class="flex flex-col gap-4 w-full max-w-5xl">
	<div class="flex justify-between items-center"><div><h2>My listings</h2><p class="text-muted-foreground text-sm">Current buy and sell orders on warframe.market</p></div><Button disabled={loading} onclick={refresh}>Refresh</Button></div>
	{#if loading}<p class="text-muted-foreground text-sm">Loading listings...</p>
	{:else if error && !editing && !removing}<p role="alert" class="text-danger text-sm">{error}</p>
	{:else if orders.length === 0}<p class="text-muted-foreground text-sm">You have no listings.</p>
	{:else}
		<div class="border border-border-secondary divide-y divide-border-secondary">
			{#each orders as order (order.id)}
				<div class="flex flex-wrap items-center gap-3 p-3 text-sm">
					<div class="flex-1 min-w-36"><div class="font-semibold">{names[order.itemId] ?? order.itemId}</div><div class="text-muted-foreground text-xs capitalize">{order.type}{order.rank != null ? ` · Rank ${order.rank}` : ''}{order.subtype ? ` · ${order.subtype}` : ''}{order.visible ? '' : ' · Hidden'}</div></div>
					<div class="tabular-nums">{order.platinum} platinum</div><div class="text-muted-foreground tabular-nums">× {order.quantity}</div>
					<Button onclick={() => edit(order)}>Edit</Button><Button onclick={() => { removing = order; error = null; }}>Remove</Button>
				</div>
			{/each}
		</div>
	{/if}
</section>

{#snippet editTitle()}Edit listing{/snippet}
{#snippet editDescription()}{editing ? names[editing.itemId] ?? editing.itemId : ''}{/snippet}
{#snippet editClose()}<Button>Cancel</Button>{/snippet}
{#snippet editActions()}<Button variant="primary" disabled={!valid || busy} onclick={save}>Save changes</Button>{/snippet}
<Dialog bind:open={() => editing !== null, (value) => { if (!value) editing = null; }} title={editTitle} description={editDescription} dialogClose={editClose} dialogActions={editActions} contentProps={{ class: 'h-auto' }}>
	<div class="grid grid-cols-2 gap-3 px-6">
		<label class="flex flex-col gap-1 text-sm">Price (platinum)<input type="number" min="1" max="900000" step="1" bind:value={price} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" /></label>
		<label class="flex flex-col gap-1 text-sm">Quantity<input type="number" min="1" max="9999" step="1" bind:value={quantity} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" /></label>
		{#if error}<p role="alert" class="col-span-2 text-danger text-sm">{error}</p>{/if}
	</div>
</Dialog>

{#snippet removeTitle()}Remove listing?{/snippet}
{#snippet removeDescription()}This removes the order from warframe.market.{/snippet}
{#snippet removeClose()}<Button>Cancel</Button>{/snippet}
{#snippet removeActions()}<Button variant="primary" disabled={busy} onclick={remove}>Remove listing</Button>{/snippet}
<Dialog bind:open={() => removing !== null, (value) => { if (!value) removing = null; }} title={removeTitle} description={removeDescription} dialogClose={removeClose} dialogActions={removeActions} contentProps={{ class: 'h-auto' }}>
	<div class="px-6 text-sm">{removing ? names[removing.itemId] ?? removing.itemId : ''}{#if error}<p role="alert" class="mt-2 text-danger">{error}</p>{/if}</div>
</Dialog>
