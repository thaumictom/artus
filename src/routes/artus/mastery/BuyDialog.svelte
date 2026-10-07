<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { z } from 'zod';
	import { untrack } from 'svelte';
	import Icon from '@iconify/svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import TableStatic from '$lib/components/TableStatic.svelte';
	import type { TableColumn } from '$lib/components/table-types';
	import CopyTradeMessage from '$lib/components/CopyTradeMessage.svelte';
	import { CachedWfmItemResponseSchema, OrderWithUserSchema } from '$lib/schemas';
	import { MarketOrdersRefresh } from '$lib/market-orders-refresh.svelte';

	type BuyItem = { slug: string; name: string };
	type Order = z.infer<typeof OrderWithUserSchema>;
	const TopOrdersSchema = z.object({ data: z.object({ sell: z.array(OrderWithUserSchema) }) });
	const columns: TableColumn[] = [
		{ key: 'name', label: 'Name' },
		{ key: 'quantity', label: 'Quantity', align: 'right' },
		{ key: 'reputation', label: 'Rep', align: 'right' },
		{ key: 'platinum', label: 'Plat', align: 'right' },
		{ key: 'copy', label: '', align: 'right' },
	];
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
		void invoke('get_cached_wfm_item', { slug })
			.then((response) => {
				if (disposed) return;
				const details = CachedWfmItemResponseSchema.parse(response).data;
				itemName = details.name || name;
				bulkTradable = details.bulkTradable ?? false;
			})
			.catch((cause) => {
				if (!disposed) console.error('Could not load buy item details:', cause);
			});
		untrack(() =>
			ordersRefresh.start(async (forceRefresh) => {
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
			}),
		);
		return () => {
			disposed = true;
			ordersRefresh.destroy();
		};
	});
</script>

{#snippet title()}Buy item{/snippet}
{#snippet description()}
	<span class="flex items-center gap-3 min-w-0">
		<span class="min-w-0 truncate" title={itemName}>{itemName}</span>
		<span class="flex items-center gap-3 whitespace-nowrap shrink-0">
			<span class="bg-border-secondary w-px h-5" aria-hidden="true"></span>
			<span class="tabular-nums">
				Median: {median === null
					? '—'
					: median.toLocaleString(undefined, { maximumFractionDigits: 1 })}
			</span>
			{#if median !== null}<img
					src="/icons/platinum.png"
					class="-ml-2 size-3.5"
					alt="platinum"
				/>{/if}
		</span>
	</span>
{/snippet}
{#snippet dialogClose()}<Button>Close</Button>{/snippet}
<Dialog
	bind:open={
		() => open,
		(value) => {
			if (!value) item = null;
		}
	}
	{title}
	{description}
	{dialogClose}
	contentProps={{ class: 'h-auto max-h-[calc(100vh-2rem)]' }}
>
	<div class="px-6 overflow-y-auto">
		<div class="flex flex-wrap justify-between items-center gap-3 mb-3">
			<h3 class="font-semibold text-base">Cheapest in-game sell orders</h3>
			<div class="flex items-center gap-3 text-muted-foreground">
				{#if ordersRefresh.fetchedAt !== null}
					<span class="tabular-nums text-base whitespace-nowrap">
						fetched {ordersRefresh.fetchedAgo}
					</span>
				{/if}
				<Button
					onclick={() => ordersRefresh.reload()}
					disabled={ordersRefresh.refreshing || ordersRefresh.coolingDown}
					class="flex items-center gap-1 text-base"
				>
					<Icon
						icon="material-symbols:refresh"
						class={ordersRefresh.refreshing ? 'size-4 animate-spin' : 'size-4'}
					/>
					{ordersRefresh.refreshing ? 'Refreshing...' : 'Reload orders'}
				</Button>
			</div>
		</div>
		{#if ordersRefresh.fetchedAt === null && !error}
			<div
				role="status"
				aria-label="Loading current orders"
				class="border border-border-secondary divide-border-secondary divide-y"
			>
				{#each Array(6) as _}
					<div class="flex items-center gap-3 px-3 py-2.5 h-12">
						<Skeleton class="flex-1 h-3" />
						<Skeleton class="w-12 h-3" />
						<Skeleton class="w-10 h-3" />
						<Skeleton class="w-10 h-3" />
						<Skeleton class="w-8 h-8" />
					</div>
				{/each}
			</div>
		{:else if error && ordersRefresh.fetchedAt === null}
			<p role="alert" class="text-danger text-base">Could not load sell orders. {error}</p>
		{:else if orders.length === 0}
			<p class="text-muted-foreground text-base">No sell orders available.</p>
		{:else}
			<TableStatic {columns} minWidth="28rem" bodyClass="divide-border-secondary divide-y">
				{#each orders as order (order.id)}
					<tr>
						<td class="px-3 py-2 max-w-0 font-medium truncate" title={order.user.ingameName}>
							{order.user.ingameName}
						</td>
						<td class="px-3 py-2 tabular-nums text-right">{order.quantity}</td>
						<td
							class="px-3 py-2 tabular-nums text-right"
							class:text-muted-foreground={order.user.reputation < 5}
						>
							{order.user.reputation}
						</td>
						<td class="px-3 py-2 font-semibold tabular-nums text-right">
							<span class="inline-flex justify-end items-center gap-1">
								{order.platinum}
								<img src="/icons/platinum.png" class="size-3.5" alt="platinum" />
							</span>
						</td>
						<td class="px-3 py-2 text-right">
							<CopyTradeMessage
								{order}
								{itemName}
								{bulkTradable}
								variantProperty={variantProperty(order)}
							/>
						</td>
					</tr>
				{/each}
			</TableStatic>
		{/if}
		{#if error && ordersRefresh.fetchedAt !== null}
			<p role="alert" class="mt-2 text-danger text-base">Could not refresh sell orders. {error}</p>
		{/if}
	</div>
</Dialog>
