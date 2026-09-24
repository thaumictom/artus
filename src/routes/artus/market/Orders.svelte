<script lang="ts">
	import Slider from '$lib/components/Slider.svelte';
	import { timeAgo } from '$lib/date';
	import { GetOrdersResponseSchema, type OrderWithUserSchema } from '$lib/schemas';
	import { RadioGroup } from 'bits-ui';
	import type z from 'zod';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount, untrack } from 'svelte';
	import Button from '$lib/components/Button.svelte';
	import { toast } from 'svelte-sonner';
	import Icon from '@iconify/svelte';
	import { createFocusedRefresh } from '$lib/focused-refresh';
	import Tooltip from '$lib/components/Tooltip.svelte';

	let {
		slug,
		itemName,
		bulkTradable = false,
		highlightSince,
		initialOrderType,
	}: {
		slug: string;
		itemName?: string;
		bulkTradable?: boolean;
		highlightSince?: number;
		initialOrderType?: 'buy' | 'sell';
	} = $props();

	const FILTER_PROPERTIES = ['rank', 'charges', 'subtype', 'amberStars', 'cyanStars'] as const;
	const REFRESH_INTERVAL_MS = 60_000;
	const MANUAL_RELOAD_COOLDOWN_MS = 3_000;
	type FilterProp = (typeof FILTER_PROPERTIES)[number];
	type Order = z.infer<typeof OrderWithUserSchema>;
	const priceFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });
	const timestampFormatter = new Intl.DateTimeFormat(undefined, {
		dateStyle: 'medium',
		timeStyle: 'medium',
	});
	const platformNames: Record<string, string> = {
		pc: 'PC',
		ps4: 'PlayStation',
		ps5: 'PlayStation',
		xbox: 'Xbox',
		switch: 'Nintendo Switch',
		mobile: 'Mobile',
	};

	function formatPlatform(platform: string) {
		return platformNames[platform.toLowerCase()] ?? platform;
	}

	function formatTimestamp(timestamp: string) {
		const date = new Date(timestamp);
		return Number.isFinite(date.getTime()) ? timestampFormatter.format(date) : 'Unknown';
	}

	function quantityPerTrade(order: Order): number {
		return bulkTradable && order.perTrade !== undefined && order.perTrade > 0 ? order.perTrade : 1;
	}

	function unitPrice(order: Order): number {
		// Bulk order platinum is the total for one bundle, not the unit price.
		return order.platinum / quantityPerTrade(order);
	}

	function tradeMessage(order: Order): string {
		const action = order.type === 'sell' ? 'buy' : 'sell';
		const quantity = quantityPerTrade(order) > 1 ? `x${quantityPerTrade(order)} ` : '';
		const variant =
			groupByProperty && order[groupByProperty] !== undefined
				? ` (${groupByProperty} ${order[groupByProperty]})`
				: '';
		return `/w ${order.user.ingameName} Hi! I want to ${action}: ${quantity}"${itemName ?? slug}${variant}" for ${order.platinum} platinum. (warframe.market)`;
	}

	let orderType = $state<'sell' | 'buy'>('sell');
	let ordersData = $state<Order[]>([]);
	let newOrderIds = $state.raw(new Set<string>());
	let previousOrderIds: Set<string> | null = null;
	let groupByProperty = $state<FilterProp | undefined>();
	let maxFilterValue = $state(0);
	let groupFilterRange = $state<[number, number]>([0, 0]);
	let fetchTimestamp = $state<number | undefined>();
	let now = $state(Date.now());
	let fetchedAgo = $derived(fetchTimestamp === undefined ? '' : timeAgo(fetchTimestamp, now));

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

	const loadOrdersData = async (
		targetSlug: string,
		targetHighlightSince: number | undefined,
		isCurrent: () => boolean,
	) => {
		try {
			const response = await invoke('get_market_orders', { slug: targetSlug });
			const { data } = GetOrdersResponseSchema.parse(response);
			if (!isCurrent()) return;

			fetchTimestamp = Date.now();
			newOrderIds =
				previousOrderIds === null
					? new Set(
							targetHighlightSince === undefined
								? []
								: data
										.filter(
											(order) =>
												(Date.parse(order.createdAt) || 0) >= targetHighlightSince,
										)
										.map((order) => order.id),
								)
							: new Set(data.filter((order) => !previousOrderIds?.has(order.id)).map((order) => order.id));
			previousOrderIds = new Set(data.map((order) => order.id));

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
				const priceDiff =
					orderType === 'sell' ? unitPrice(a) - unitPrice(b) : unitPrice(b) - unitPrice(a);
				if (priceDiff !== 0) return priceDiff;

				// Tie-breaker: newest updatedAt first (descending)
				const ta = Date.parse(a.updatedAt) || 0;
				const tb = Date.parse(b.updatedAt) || 0;
				return tb - ta;
			});
	});

	// Each mounted item owns its refresh lifecycle.
	$effect(() => {
		const targetSlug = slug;
		const targetHighlightSince = highlightSince;
		const targetOrderType = initialOrderType;
		if (!targetSlug) return;
		let disposed = false;
		let cooldownTimer: ReturnType<typeof setTimeout> | undefined;

		untrack(() => {
			isRefreshing = false;
			isReloadCoolingDown = false;
			ordersError = null;
			ordersData = [];
			newOrderIds = new Set();
			previousOrderIds = null;
			orderType = targetOrderType ?? 'sell';
			fetchTimestamp = undefined;
			groupByProperty = undefined;
			maxFilterValue = 0;
			groupFilterRange = [0, 0];
		});

		const focusedRefresh = createFocusedRefresh(async () => {
			isRefreshing = true;
			ordersError = null;
			try {
				await loadOrdersData(targetSlug, targetHighlightSince, () => !disposed);
			} finally {
				if (!disposed) {
					isRefreshing = false;
				}
			}
		}, REFRESH_INTERVAL_MS, { immediate: true });

		reloadOrders = () => {
			if (disposed || isRefreshing || isReloadCoolingDown) return;
			isReloadCoolingDown = true;
			cooldownTimer = setTimeout(() => {
				isReloadCoolingDown = false;
			}, MANUAL_RELOAD_COOLDOWN_MS);
			void focusedRefresh.refresh();
		};
		return () => {
			disposed = true;
			clearTimeout(cooldownTimer);
			focusedRefresh.destroy();
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

<div class="flex flex-col gap-4 w-full max-w-3xl">
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
					<td>
						<div class="flex items-center gap-2">
							{#if newOrderIds.has(order.id)}
								<span
									class="bg-success rounded-full size-2 shrink-0"
									title="New since the previous fetch"
									aria-label="New order"
								></span>
							{/if}
							<span
								title={formatPlatform(order.user.platform)}
								aria-label={`${formatPlatform(order.user.platform)} platform`}
							>
								<Icon
									icon={order.user.platform.toLowerCase() === 'pc'
										? 'material-symbols:computer-outline-rounded'
										: order.user.platform.toLowerCase() === 'mobile'
											? 'material-symbols:smartphone-outline-rounded'
											: 'material-symbols:sports-esports-outline-rounded'}
									class="size-4 text-muted-foreground"
								/>
							</span>
							<span>{order.user.ingameName}</span>
						</div>
					</td>
					<td align="right" class={{ 'text-muted-foreground': order.user.reputation < 5 }}>
						<span>{order.user.reputation}</span>
					</td>
					<td align="right">
						<div class="flex justify-end items-center gap-1">
							<span>{priceFormatter.format(unitPrice(order))}</span>
							<img src="/icons/platinum.png" alt="Platinum" class="size-3" />
						</div>
						{#if quantityPerTrade(order) > 1}
							<div class="text-muted-foreground text-xs">
								{priceFormatter.format(order.platinum)} total
							</div>
						{/if}
					</td>
					<td align="right">
						<div>{order.quantity}</div>
						{#if quantityPerTrade(order) > 1}
							<div class="text-muted-foreground text-xs">{quantityPerTrade(order)} per trade</div>
						{/if}
					</td>
					{#if groupByProperty}
						<td align="right">
							{order[groupByProperty]}{#if maxFilterValue !== undefined && maxFilterValue > 0}
								&nbsp;of {maxFilterValue}{/if}
						</td>
					{/if}
					<td align="right" class="py-0!">
						<Tooltip
							side="left"
							align="center"
							class="hover:bg-surface p-1 border cursor-pointer"
							triggerProps={{
								'aria-label': 'Copy trade message to clipboard',
								onclick: () => copyToClipboard(tradeMessage(order)),
							}}
						>
							{#snippet children()}
								<Icon icon="material-symbols:content-copy" class="size-4" />
							{/snippet}
							{#snippet content()}
								<div class="whitespace-nowrap text-xs">
									<div class="mb-2 font-medium">Copy to clipboard</div>
									<div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
										<span class="text-muted-foreground">Platform</span>
										<span>{formatPlatform(order.user.platform)}</span>
										{#if order.user.country}
											<span class="text-muted-foreground">Country</span>
											<span>{order.user.country}</span>
										{/if}
										<span class="text-muted-foreground">Language</span>
										<span>{order.user.locale}</span>
										<span class="text-muted-foreground">Created</span>
										<time datetime={order.createdAt}>{formatTimestamp(order.createdAt)}</time>
										<span class="text-muted-foreground">Updated</span>
										<time datetime={order.updatedAt}>{formatTimestamp(order.updatedAt)}</time>
									</div>
								</div>
							{/snippet}
						</Tooltip>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
