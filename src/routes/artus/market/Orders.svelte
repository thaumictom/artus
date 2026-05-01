<script lang="ts">
	import Slider from '$lib/components/Slider.svelte';
	import { GetOrdersResponseSchema, type OrderWithUserSchema } from '$lib/schemas';
	import { RadioGroup } from 'bits-ui';
	import type z from 'zod';
	import { invoke } from '@tauri-apps/api/core';

	let { slug, itemName }: { slug: string; itemName?: string } = $props();

	const FILTER_PROPERTIES = ['rank', 'charges', 'subtype', 'amberStars', 'cyanStars'] as const;
	type FilterProp = (typeof FILTER_PROPERTIES)[number];

	// State
	let orderType = $state<'sell' | 'buy'>('sell');
	let ordersData = $state<z.infer<typeof OrderWithUserSchema>[]>([]);
	let groupByProperty = $state<FilterProp | undefined>();
	let maxFilterValue = $state(0);
	let groupFilterRange = $state<[number, number]>([0, 0]);
	let fetchTimestamp = $state<number | undefined>();

	const loadOrdersData = async (targetSlug: string) => {
		try {
			const response = await invoke('get_market_orders', { slug: targetSlug });
			const { data } = GetOrdersResponseSchema.parse(response);

			fetchTimestamp = Date.now();

			ordersData = data;

			// Auto-detect property and set initial slider bounds
			const firstItem = data[0];
			if (firstItem) {
				const foundProp = FILTER_PROPERTIES.find((p) => p in firstItem);
				if (foundProp) {
					groupByProperty = foundProp;
					const val = firstItem[foundProp];
					if (typeof val === 'number') {
						const max = Math.max(...data.map((i) => Number(i[foundProp]) || 0));
						maxFilterValue = max;
						groupFilterRange = [0, max];
					}
				}
			}
		} catch (err) {
			console.error('Failed to load orders:', err);
		}
	};

	// Derived filtering & sorting
	let filteredOrders = $derived.by(() => {
		return ordersData
			.filter((o) => {
				const typeMatch = o.type === orderType;
				if (!typeMatch) return false;
				if (o.user?.status !== 'ingame') return false;
				if (!groupByProperty) return true;

				const val = o[groupByProperty];
				if (typeof val !== 'number') return true;

				return val >= groupFilterRange[0] && val <= groupFilterRange[1];
			})
			.sort((a, b) => {
				const priceDiff = orderType === 'sell' ? a.platinum - b.platinum : b.platinum - a.platinum;
				if (priceDiff !== 0) return priceDiff;

				// Tie-breaker: newest updatedAt first (descending)
				const ta = Date.parse(a.updatedAt as string) || 0;
				const tb = Date.parse(b.updatedAt as string) || 0;
				return tb - ta;
			});
	});

	// Handle slug changes
	$effect(() => {
		if (slug) loadOrdersData(slug);
	});

	import { toast } from 'svelte-sonner';
	import Icon from '@iconify/svelte';

	// Copy to clipboard, toast on success and error
	const copyToClipboard = async (text: string) => {
		try {
			await navigator.clipboard.writeText(text);
			// Show success toast
			toast.success('Copied to clipboard');
		} catch (err) {
			console.error('Failed to copy:', err);
			// Show error toast
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
	<div class="flex justify-between items-center text-muted-foreground">
		{#if fetchTimestamp}
			<div class="text-sm">
				fetched orders at: {new Date(fetchTimestamp).toLocaleString(undefined, { hour12: false })}
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
	<table class="border-collapse">
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
							<span>{order.platinum}</span>
							<img src="/icons/platinum.png" alt="Platinum" class="size-3" />
						</div>
					</td>
					<td align="right">{order.quantity}</td>
					{#if groupByProperty}
						<td align="right">{order[groupByProperty]} of {maxFilterValue}</td>
					{/if}
					<td align="right" class="py-0!">
						<button
							class="hover:bg-surface p-1 border cursor-pointer"
							onclick={() =>
								copyToClipboard(
									`/w ${order.user.ingameName} Hi! I want to ${order.type === 'sell' ? 'buy' : 'sell'}: "${itemName}${groupByProperty ? ` (${groupByProperty} ${order[groupByProperty]})` : ''}" for ${order.platinum} platinum. (warframe.market)`,
								)}
						>
							<Icon icon="material-symbols:content-copy" class="size-4" />
						</button>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
