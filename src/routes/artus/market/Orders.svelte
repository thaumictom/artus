<script lang="ts">
	import Slider from '$lib/components/Slider.svelte';
	import { GetOrdersResponseSchema, type OrderWithUserSchema } from '$lib/schemas';
	import { RadioGroup } from 'bits-ui';
	import type z from 'zod';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount, untrack } from 'svelte';
	import Button from '$lib/components/Button.svelte';
	import { toast } from 'svelte-sonner';
	import Icon from '@iconify/svelte';

	let { slug, itemName, bulkTradable = false }: {
		slug: string;
		itemName?: string;
		bulkTradable?: boolean;
	} = $props();

	const FILTER_PROPERTIES = ['rank', 'charges', 'subtype', 'amberStars', 'cyanStars'] as const;
	const REFRESH_INTERVAL_MS = 60_000;
	const MANUAL_RELOAD_COOLDOWN_MS = 3_000;
	type FilterProp = (typeof FILTER_PROPERTIES)[number];
	type Order = z.infer<typeof OrderWithUserSchema>;
	const priceFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });

	function quantityPerTrade(order: Order): number {
		return bulkTradable && order.perTrade !== undefined && order.perTrade > 0
			? order.perTrade
			: 1;
	}

	function unitPrice(order: Order): number {
		// Bulk order platinum is the total for one bundle, not the unit price.
		return order.platinum / quantityPerTrade(order);
	}

	function tradeMessage(order: Order): string {
		const action = order.type === 'sell' ? 'buy' : 'sell';
		const quantity = quantityPerTrade(order) > 1 ? `x${quantityPerTrade(order)} ` : '';
		const variant = groupByProperty && order[groupByProperty] !== undefined
			? ` (${groupByProperty} ${order[groupByProperty]})`
			: '';
		return `/w ${order.user.ingameName} Hi! I want to ${action}: ${quantity}"${itemName ?? slug}${variant}" for ${order.platinum} platinum. (warframe.market)`;
	}

	let orderType = $state<'sell' | 'buy'>('sell');
	let ordersData = $state<Order[]>([]);
	let groupByProperty = $state<FilterProp | undefined>();
	let maxFilterValue = $state(0);
	let groupFilterRange = $state<[number, number]>([0, 0]);
	let fetchTimestamp = $state<number | undefined>();
	let now = $state(Date.now());
	const relativeTime = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });
	let fetchedAgo = $derived.by(() => {
		if (fetchTimestamp === undefined) return '';
		const seconds = Math.max(0, Math.floor((now - fetchTimestamp) / 1000));
		if (seconds < 3) return 'just now';
		if (seconds < 60) return relativeTime.format(-seconds, 'second');
		if (seconds < 3600) return relativeTime.format(-Math.floor(seconds / 60), 'minute');
		if (seconds < 86400) return relativeTime.format(-Math.floor(seconds / 3600), 'hour');
		return relativeTime.format(-Math.floor(seconds / 86400), 'day');
	});

	onMount(() => {
		const timer = setInterval(() => {
			now = Date.now();
		}, 1000);
		return () => clearInterval(timer);
	});
	let isRefreshing = $state(false);
	let isReloadCoolingDown = $state(false);
	let ordersError = $state<string | null>(null);
	let reloadOrders = () => {};

	function updateFilterBounds(data: Order[]) {
		const firstItem = data[0];
		if (!firstItem) return;
		const property = FILTER_PROPERTIES.find((key) => key in firstItem);
		if (!property) return;

		const sameProperty = groupByProperty === property;
		const fullRange = groupFilterRange[0] === 0 && groupFilterRange[1] === maxFilterValue;
		groupByProperty = property;
		if (typeof firstItem[property] !== 'number') return;

		const max = Math.max(...data.map((order) => Number(order[property]) || 0));
		maxFilterValue = sameProperty ? Math.max(maxFilterValue, max) : max;
		// Preserve a custom selection while allowing an unfiltered range to expand.
		if (!sameProperty || fullRange) groupFilterRange = [0, maxFilterValue];
	}

	const loadOrdersData = async (targetSlug: string, isCurrent: () => boolean) => {
		try {
			const response = await invoke('get_market_orders', { slug: targetSlug });
			const { data } = GetOrdersResponseSchema.parse(response);
			if (!isCurrent()) return;

			fetchTimestamp = Date.now();

			ordersData = data;

			updateFilterBounds(data);
		} catch (err) {
			if (!isCurrent()) return;
			console.error('Failed to load orders:', err);
			ordersError = 'Could not refresh orders. Please try again.';
		}
	};

	// Derived filtering & sorting
	let filteredOrders = $derived.by(() => {
		return ordersData
			.filter((o) => {
				if (o.type !== orderType || o.user.status !== 'ingame') return false;
				if (!groupByProperty) return true;

				const val = o[groupByProperty];
				if (typeof val !== 'number') return true;

				return val >= groupFilterRange[0] && val <= groupFilterRange[1];
			})
			.sort((a, b) => {
				const priceDiff = orderType === 'sell' ? unitPrice(a) - unitPrice(b) : unitPrice(b) - unitPrice(a);
				if (priceDiff !== 0) return priceDiff;

				// Tie-breaker: newest updatedAt first (descending)
				const ta = Date.parse(a.updatedAt) || 0;
				const tb = Date.parse(b.updatedAt) || 0;
				return tb - ta;
			});
	});

	// Each mounted item owns its timer and listeners. An overdue refresh waits for focus.
	$effect(() => {
		const targetSlug = slug;
		if (!targetSlug) return;
		let disposed = false;
		let inFlight = false;
		let nextRefreshAt = 0;
		let timer: ReturnType<typeof setTimeout> | undefined;
		let cooldownTimer: ReturnType<typeof setTimeout> | undefined;

		const refresh = async () => {
			if (disposed || inFlight) return;
			inFlight = true;
			isRefreshing = true;
			ordersError = null;
			clearTimeout(timer);
			try {
				await loadOrdersData(targetSlug, () => !disposed);
			} finally {
				if (!disposed) {
					inFlight = false;
					isRefreshing = false;
					// Reset after manual refreshes and failed requests as well, avoiding retry loops.
					nextRefreshAt = Date.now() + REFRESH_INTERVAL_MS;
					timer = setTimeout(refreshIfDue, REFRESH_INTERVAL_MS);
				}
			}
		};
		const refreshIfDue = () => {
			if (document.hasFocus() && !document.hidden && Date.now() >= nextRefreshAt) {
				void refresh();
			}
		};
		reloadOrders = () => {
			if (disposed || inFlight || isReloadCoolingDown) return;
			isReloadCoolingDown = true;
			cooldownTimer = setTimeout(() => {
				isReloadCoolingDown = false;
			}, MANUAL_RELOAD_COOLDOWN_MS);
			void refresh();
		};
		window.addEventListener('focus', refreshIfDue);
		document.addEventListener('visibilitychange', refreshIfDue);
		untrack(() => {
			isRefreshing = false;
			isReloadCoolingDown = false;
			ordersError = null;
			ordersData = [];
			fetchTimestamp = undefined;
			groupByProperty = undefined;
			maxFilterValue = 0;
			groupFilterRange = [0, 0];
			refreshIfDue();
		});
		return () => {
			disposed = true;
			clearTimeout(timer);
			clearTimeout(cooldownTimer);
			window.removeEventListener('focus', refreshIfDue);
			document.removeEventListener('visibilitychange', refreshIfDue);
		};
	});

	const copyToClipboard = async (text: string) => {
		try {
			await navigator.clipboard.writeText(text);
			toast.success('Copied to clipboard');
		} catch (err) {
			console.error('Failed to copy:', err);
			toast.error('Failed to copy to clipboard');
		}
	};
</script>

<div class="flex flex-col gap-4 w-full max-w-2xl">
	<div class="flex items-center gap-4 w-full">
		{#if groupByProperty && maxFilterValue > 0}
			<div class="flex flex-col gap-1 grow">
				<div class="flex justify-between text-xs">
					<div>{groupByProperty} filter</div>
					<div class="text-muted-foreground">
						{#if groupFilterRange[0] === groupFilterRange[1]}
							<span>{groupFilterRange[0]}</span>
						{:else}
							<span>{groupFilterRange[0]} - {groupFilterRange[1]}</span>
						{/if}
					</div>
				</div>
				<Slider
					type="multiple"
					min={0}
					max={maxFilterValue}
					step={1}
					bind:value={groupFilterRange}
				/>
			</div>
		{/if}
		<RadioGroup.Root class="flex select-none shrink-0" bind:value={orderType}>
			<div class="p-1 border">
				<div
					class="inline-flex gap-1 *:data-[state=checked]:bg-surface *:px-4 *:py-1 *:cursor-pointer"
				>
					<RadioGroup.Item value="sell">Sell Orders</RadioGroup.Item>
					<RadioGroup.Item value="buy">Buy Orders</RadioGroup.Item>
				</div>
			</div>
		</RadioGroup.Root>
		{#if !groupByProperty}
			<div class="bg-surface w-full h-px"></div>
		{/if}
	</div>
	<div class="flex flex-wrap justify-between items-center gap-2 text-muted-foreground">
		<Button
			onclick={() => reloadOrders()}
			disabled={isRefreshing || isReloadCoolingDown}
			class="flex items-center gap-1 text-sm"
		>
			<Icon
				icon="material-symbols:refresh"
				class={isRefreshing ? 'size-4 animate-spin' : 'size-4'}
			/>
			{isRefreshing ? 'Refreshing...' : 'Reload orders'}
		</Button>
		{#if fetchTimestamp}
			<div class="tabular-nums text-sm">
				fetched {fetchedAgo}
			</div>
		{/if}
		<a
			href={`https://warframe.market/items/${slug}`}
			target="_blank"
			class="text-sm hover:underline"
		>
			<span>view on warframe.market</span>
			<Icon icon="material-symbols:arrow-outward-rounded" class="inline size-4" />
		</a>
	</div>
	{#if ordersError}
		<p role="alert" class="text-danger text-sm">{ordersError}</p>
	{/if}
	<table class="border-collapse" aria-busy={isRefreshing}>
		<thead>
			<tr class="*:px-1 *:py-2">
				<th scope="col" align="left">Name</th>
				<th scope="col" align="right">Reputation</th>
				<th scope="col" align="right">Platinum</th>
				<th scope="col" align="right">Quantity</th>
				{#if groupByProperty}
					<th scope="col" align="right">
						{groupByProperty.charAt(0).toUpperCase() + groupByProperty.slice(1)}
					</th>
				{/if}
				<th scope="col"></th>
			</tr>
		</thead>
		<tbody>
			{#each filteredOrders as order (order.id)}
				<tr class="*:px-1 *:py-2 *:border-t">
					<td>{order.user.ingameName}</td>
					<td align="right" class={{ 'text-muted-foreground': order.user.reputation < 5 }}>
						<span>{order.user.reputation}</span>
					</td>
					<td align="right">
						<div class="flex justify-end items-center gap-1">
							<span>{priceFormatter.format(unitPrice(order))}</span>
							<img src="/icons/platinum.png" alt="Platinum" class="size-3" />
						</div>
						{#if quantityPerTrade(order) > 1}
							<div class="text-muted-foreground text-xs">{priceFormatter.format(order.platinum)} total</div>
						{/if}
					</td>
					<td align="right">
						<div>{order.quantity}</div>
						{#if quantityPerTrade(order) > 1}
							<div class="text-muted-foreground text-xs">{quantityPerTrade(order)} per trade</div>
						{/if}
					</td>
					{#if groupByProperty}
						<td align="right">{order[groupByProperty]} of {maxFilterValue}</td>
					{/if}
					<td align="right" class="py-0!">
						<button
							class="hover:bg-surface p-1 border cursor-pointer"
							onclick={() => copyToClipboard(tradeMessage(order))}
						>
							<Icon icon="material-symbols:content-copy" class="size-4" />
						</button>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
