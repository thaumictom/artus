<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { z } from 'zod';
	import { untrack } from 'svelte';
	import Icon from '@iconify/svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import CopyTradeMessage from '$lib/components/CopyTradeMessage.svelte';
	import { CachedWfmItemResponseSchema, OrderWithUserSchema } from '$lib/schemas';
	import { MarketOrdersRefresh } from '$lib/market-orders-refresh.svelte';

	type BuyItem = { slug: string; name: string };
	type Order = z.infer<typeof OrderWithUserSchema>;
	const TopOrdersSchema = z.object({ data: z.object({ sell: z.array(OrderWithUserSchema) }) });
	const variantProperties = ['rank', 'charges', 'subtype', 'amberStars', 'cyanStars'] as const;
	let { item = $bindable<BuyItem | null>(null) }: { item: BuyItem | null } = $props();
	let open = $derived(item !== null);
	let orders = $state<Order[]>([]);
	let itemName = $state('');
	let bulkTradable = $state(false);
	let error = $state<string | null>(null);
	const ordersRefresh = new MarketOrdersRefresh();
	const median = $derived.by(() => {
		if (orders.length === 0) return null;
		const prices = orders.map((order) => order.platinum).sort((a, b) => a - b);
		const middle = Math.floor(prices.length / 2);
		return prices.length % 2 ? prices[middle] : (prices[middle - 1] + prices[middle]) / 2;
	});

	function variantProperty(order: Order) {
		return variantProperties.find((key) => order[key] !== undefined);
	}

	$effect(() => {
		if (!item) return;
		const { slug, name } = item;
		let disposed = false;
		untrack(() => {
			orders = [];
			itemName = name;
			bulkTradable = false;
			error = null;
		});
		void invoke('get_cached_wfm_item', { slug }).then((response) => {
			if (disposed) return;
			const details = CachedWfmItemResponseSchema.parse(response).data;
			itemName = details.name || name;
			bulkTradable = details.bulkTradable ?? false;
		}).catch((cause) => {
			if (!disposed) console.error('Could not load buy item details:', cause);
		});
		untrack(() => ordersRefresh.start(async (forceRefresh) => {
			error = null;
			try {
				const response = await invoke('market_top_orders', { slug, forceRefresh });
				if (disposed) return false;
				orders = TopOrdersSchema.parse(response).data.sell;
				return true;
			} catch (cause) {
				if (!disposed) error = String(cause);
				return false;
			}
		}));
		return () => {
			disposed = true;
			ordersRefresh.destroy();
		};
	});
</script>

{#snippet title()}Buy item{/snippet}
{#snippet description()}{item?.name ?? ''}{/snippet}
{#snippet dialogClose()}<Button>Close</Button>{/snippet}
<Dialog bind:open={() => open, (value) => { if (!value) item = null; }} {title} {description} {dialogClose}
	contentProps={{ class: 'h-auto max-h-[calc(100vh-2rem)]' }}>
	<div class="px-6 overflow-y-auto">
		<div class="flex flex-wrap justify-between items-center gap-3 mb-3">
			<h3 class="font-semibold text-sm">Cheapest in-game sell orders</h3>
			<div class="flex items-center gap-3 text-muted-foreground">
				<Button onclick={() => ordersRefresh.reload()} disabled={ordersRefresh.refreshing || ordersRefresh.coolingDown} class="flex items-center gap-1 text-sm">
					<Icon icon="material-symbols:refresh" class={ordersRefresh.refreshing ? 'size-4 animate-spin' : 'size-4'} />
					{ordersRefresh.refreshing ? 'Refreshing...' : 'Reload orders'}
				</Button>
				{#if ordersRefresh.fetchedAt !== null}
					<span class="tabular-nums text-sm whitespace-nowrap">fetched {ordersRefresh.fetchedAgo}</span>
				{/if}
			</div>
		</div>
		{#if ordersRefresh.fetchedAt === null && !error}
			<p class="text-muted-foreground text-sm">Loading current orders...</p>
		{:else if error && ordersRefresh.fetchedAt === null}
			<p role="alert" class="text-danger text-sm">Could not load sell orders. {error}</p>
		{:else if orders.length === 0}
			<p class="text-muted-foreground text-sm">No sell orders available.</p>
		{:else}
			<div class="border border-border-secondary divide-y divide-border-secondary">
				{#each orders as order (order.id)}
					<div class="flex items-center gap-3 px-3 py-2 text-sm">
						<div class="min-w-0 flex-1">
							<div class="truncate font-medium">{order.user.ingameName}</div>
							<div class="text-muted-foreground text-xs">{order.user.status} · {order.quantity} available</div>
						</div>
						<div class="flex items-center gap-1 tabular-nums font-semibold"><span>{order.platinum}</span><img src="/icons/platinum.png" class="size-3.5" alt="platinum" /></div>
						<CopyTradeMessage {order} {itemName} {bulkTradable} variantProperty={variantProperty(order)} />
					</div>
				{/each}
				<div class="flex items-center justify-between px-3 py-2 text-sm font-semibold">
					<span>Median sell price</span>
					<span class="flex items-center gap-1 tabular-nums">{median}<img src="/icons/platinum.png" class="size-3.5" alt="platinum" /></span>
				</div>
			</div>
		{/if}
		{#if error && ordersRefresh.fetchedAt !== null}
			<p role="alert" class="mt-2 text-danger text-sm">Could not refresh sell orders. {error}</p>
		{/if}
	</div>
</Dialog>
