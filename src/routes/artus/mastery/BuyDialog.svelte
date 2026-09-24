<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { z } from 'zod';
	import Dialog from '$lib/components/Dialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import CopyTradeMessage from '$lib/components/CopyTradeMessage.svelte';
	import { GetItemResponseSchema, OrderWithUserSchema } from '$lib/schemas';

	type BuyItem = { slug: string; name: string };
	type Order = z.infer<typeof OrderWithUserSchema>;
	const TopOrdersSchema = z.object({ data: z.object({ sell: z.array(OrderWithUserSchema) }) });
	const variantProperties = ['rank', 'charges', 'subtype', 'amberStars', 'cyanStars'] as const;
	let { item = $bindable<BuyItem | null>(null) }: { item: BuyItem | null } = $props();
	let open = $derived(item !== null);
	let orders = $state<Order[]>([]);
	let itemName = $state('');
	let bulkTradable = $state(false);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let requestId = 0;
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
		const current = ++requestId;
		orders = [];
		itemName = name;
		bulkTradable = false;
		error = null;
		loading = true;
		void Promise.all([
			invoke('market_top_orders', { slug }),
			invoke('get_market_item', { slug }),
		]).then(([topResponse, itemResponse]) => {
			if (current !== requestId) return;
			orders = TopOrdersSchema.parse(topResponse).data.sell.slice(0, 5);
			const details = GetItemResponseSchema.parse(itemResponse).data;
			itemName = details.i18n?.en?.name ?? name;
			bulkTradable = details.bulkTradable ?? false;
		}).catch((cause) => {
			if (current === requestId) error = String(cause);
		}).finally(() => {
			if (current === requestId) loading = false;
		});
		return () => { requestId++; };
	});
</script>

{#snippet title()}Buy item{/snippet}
{#snippet description()}{item?.name ?? ''}{/snippet}
{#snippet dialogClose()}<Button>Close</Button>{/snippet}
<Dialog bind:open={() => open, (value) => { if (!value) item = null; }} {title} {description} {dialogClose}
	contentProps={{ class: 'h-auto max-h-[calc(100vh-2rem)]' }}>
	<div class="px-6 overflow-y-auto">
		<h3 class="mb-3 font-semibold text-sm">Top sell orders</h3>
		{#if loading}
			<p class="text-muted-foreground text-sm">Loading current orders...</p>
		{:else if error}
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
	</div>
</Dialog>
