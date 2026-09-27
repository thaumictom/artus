<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { untrack } from 'svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import type { InventoryItem } from '$lib/inventory';
	import { GetOrdersResponseSchema } from '$lib/schemas';
	import { config } from '$lib/settings.svelte';
	import Keybind from '$lib/components/Keybind.svelte';
	import ListingItemInfo from '$lib/components/ListingItemInfo.svelte';
	import ListingQuantityWarning from '$lib/components/ListingQuantityWarning.svelte';
	import type { EditableListing } from '../listings/types';
	type ListingItemDetails = {
		maxRank?: number;
		maxCharges?: number;
		maxAmberStars?: number;
		maxCyanStars?: number;
		subtypes?: string[];
		bulkTradable?: boolean;
	};
	type OrderSide = 'sell' | 'buy';
	type ListingControl =
		| 'price'
		| 'quantity'
		| 'rank'
		| 'charges'
		| 'amberStars'
		| 'cyanStars'
		| 'subtype'
		| 'cancel'
		| 'hidden'
		| 'visible';
	type OrderPreview = { id: string; platinum: number; quantity: number };
	type TodayStatistics = {
		median: number | null;
		weightedAverage: number | null;
		volume: number | null;
	};
	const priceFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });
	const volumeFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });

	function selectTopOrders(
		orders: (OrderPreview & { type: OrderSide; user: { status: string } })[],
		side: OrderSide,
	): OrderPreview[] {
		return orders
			.filter(
				(order) =>
					order.type === side &&
					order.user.status === 'ingame' &&
					Number.isSafeInteger(order.platinum) &&
					order.platinum > 0 &&
					order.quantity > 0,
			)
			.map(({ id, platinum, quantity }) => ({ id, platinum, quantity }))
			.sort((a, b) => (side === 'sell' ? a.platinum - b.platinum : b.platinum - a.platinum))
			.slice(0, 10);
	}

	let {
		item = $bindable<InventoryItem | null>(null),
		mastered = false,
		onSaved = () => {},
		editing = null,
		overlayMode = false,
		overlayHotkey = null,
		initialSubtype,
	}: {
		item: InventoryItem | null;
		mastered?: boolean;
		onSaved?: () => void;
		editing?: EditableListing | null;
		overlayMode?: boolean;
		overlayHotkey?: { action: string; sequence: number } | null;
		initialSubtype?: string;
	} = $props();
	let open = $derived(item !== null);
	let isEditing = $derived(editing !== null);
	let dialogTitle = $derived(
		editing?.type === 'sell'
			? 'Edit sell listing'
			: editing?.type === 'buy'
				? 'Edit buy order'
				: 'Create sell listing',
	);
	let sellOrders = $state<OrderPreview[]>([]);
	let buyOrders = $state<OrderPreview[]>([]);
	let lowestSell = $derived(sellOrders[0]?.platinum ?? null);
	let highestBuy = $derived(buyOrders[0]?.platinum ?? null);
	let spread = $derived(
		lowestSell !== null && highestBuy !== null ? lowestSell - highestBuy : null,
	);
	let todayStatistics = $state<TodayStatistics | null>(null);
	let loading = $state(false);
	let busy = $state(false);
	let error = $state<string | null>(null);
	let price = $state(1);
	let quantity = $state(1);
	let details = $state<ListingItemDetails | null>(null);
	let rank = $state(0);
	let charges = $state(0);
	let amberStars = $state(0);
	let cyanStars = $state(0);
	let subtype = $state('');
	let requestId = 0;
	let focusedControl = $state<ListingControl>('price');
	const listingControls = $derived<ListingControl[]>([
		'price',
		'quantity',
		...(!isEditing && details?.maxRank ? ['rank' as const] : []),
		...(!isEditing && details?.maxCharges ? ['charges' as const] : []),
		...(!isEditing && details?.maxAmberStars ? ['amberStars' as const] : []),
		...(!isEditing && details?.maxCyanStars ? ['cyanStars' as const] : []),
		...(!isEditing && details?.subtypes?.length ? ['subtype' as const] : []),
		'cancel',
		...(!isEditing ? ['hidden' as const] : []),
		'visible',
	]);
	const selectedClass = (control: ListingControl) =>
		overlayMode && focusedControl === control ? 'overlay-control-selected' : '';

	function cycleListingControl(direction: 1 | -1) {
		const index = listingControls.indexOf(focusedControl);
		focusedControl =
			listingControls[(index + direction + listingControls.length) % listingControls.length];
	}

	function adjustListingControl(direction: 1 | -1) {
		switch (focusedControl) {
			case 'price':
				price = Math.min(900000, Math.max(1, price + direction));
				break;
			case 'quantity':
				quantity = Math.min(9999, Math.max(1, quantity + direction));
				break;
			case 'rank':
				rank = Math.min(details?.maxRank ?? 0, Math.max(0, rank + direction));
				break;
			case 'charges':
				charges = Math.min(details?.maxCharges ?? 0, Math.max(0, charges + direction));
				break;
			case 'amberStars':
				amberStars = Math.min(details?.maxAmberStars ?? 0, Math.max(0, amberStars + direction));
				break;
			case 'cyanStars':
				cyanStars = Math.min(details?.maxCyanStars ?? 0, Math.max(0, cyanStars + direction));
				break;
			case 'subtype': {
				const options = details?.subtypes ?? [];
				if (options.length > 0) {
					const index = options.indexOf(subtype);
					subtype = options[(index + direction + options.length) % options.length];
				}
				break;
			}
		}
	}

	function handleListingHotkey(action: string) {
		if (action === 'listing_cancel') {
			item = null;
			return;
		}
		if (action === 'cycle' || action === 'navigate_right') {
			cycleListingControl(1);
			return;
		}
		if (action === 'cycle_back' || action === 'navigate_left') {
			cycleListingControl(-1);
			return;
		}
		if (action === 'navigate_up') {
			adjustListingControl(1);
			return;
		}
		if (action === 'navigate_down') {
			adjustListingControl(-1);
			return;
		}
		if (action !== 'listing_confirm') return;
		if (focusedControl === 'cancel') item = null;
		else if (focusedControl === 'hidden') void save(false);
		else if (focusedControl === 'visible') void save(true);
		else cycleListingControl(1);
	}

	$effect(() => {
		const hotkey = overlayHotkey;
		if (overlayMode && item && hotkey) untrack(() => handleListingHotkey(hotkey.action));
	});
	$effect(() => {
		if (!overlayMode || !open) return;
		// The game shortcut handles Escape while Warframe is focused; this covers DOM focus in the dialog.
		const onKeyDown = (event: KeyboardEvent) => {
			if (event.key !== 'Escape') return;
			event.preventDefault();
			event.stopImmediatePropagation();
			item = null;
		};
		window.addEventListener('keydown', onKeyDown, true);
		return () => window.removeEventListener('keydown', onKeyDown, true);
	});
	const variantValid = $derived(
		(!details?.maxRank || (Number.isSafeInteger(rank) && rank >= 0 && rank <= details.maxRank)) &&
			(!details?.maxCharges ||
				(Number.isSafeInteger(charges) && charges >= 0 && charges <= details.maxCharges)) &&
			(!details?.maxAmberStars ||
				(Number.isSafeInteger(amberStars) &&
					amberStars >= 0 &&
					amberStars <= details.maxAmberStars)) &&
			(!details?.maxCyanStars ||
				(Number.isSafeInteger(cyanStars) && cyanStars >= 0 && cyanStars <= details.maxCyanStars)) &&
			(!details?.subtypes?.length || details.subtypes.includes(subtype)),
	);
	const valid = $derived(
		Number.isSafeInteger(price) &&
			price >= 1 &&
			price <= 900000 &&
			Number.isSafeInteger(quantity) &&
			quantity >= 1 &&
			quantity <= 9999 &&
			(isEditing || variantValid),
	);
	$effect(() => {
		if (!item?.slug) return;
		const slug = item.slug;
		quantity = editing?.quantity ?? Math.min(Math.max(item.quantity, 1), 9999);
		price = editing?.platinum ?? 1;
		focusedControl = 'price';
		sellOrders = [];
		buyOrders = [];
		todayStatistics = null;
		details = null;
		error = null;
		loading = true;
		const current = ++requestId;
		void Promise.all([
			invoke('get_market_orders', { slug }),
			invoke<{ data: ListingItemDetails }>('get_market_item', { slug }),
			invoke<TodayStatistics | null>('get_tradeable_today_statistics', { slug }),
		])
			.then(([response, itemResponse, statistics]) => {
				if (current !== requestId) return;
				const orders = GetOrdersResponseSchema.parse(response).data;
				sellOrders = selectTopOrders(orders, 'sell');
				buyOrders = selectTopOrders(orders, 'buy');
				todayStatistics = statistics;
				if (!editing) price = sellOrders[0]?.platinum ?? 1;
				details = itemResponse.data;
				rank = 0;
				charges = 0;
				amberStars = 0;
				cyanStars = 0;
				subtype =
					itemResponse.data?.subtypes?.find(
						(option) => option.toLowerCase() === initialSubtype?.toLowerCase(),
					) ??
					itemResponse.data?.subtypes?.[0] ??
					'';
			})
			.catch((cause) => {
				if (current === requestId) error = String(cause);
			})
			.finally(() => {
				if (current === requestId) loading = false;
			});
	});

	async function save(visible: boolean) {
		if (!item?.slug || !valid || busy) return;
		busy = true;
		error = null;
		try {
			if (editing) {
				await invoke('market_update_listing', { id: editing.id, platinum: price, quantity });
			} else {
				await invoke('market_create_listing', {
					slug: item.slug,
					platinum: price,
					quantity,
					visible,
					variant: {
						rank: details?.maxRank ? rank : null,
						charges: details?.maxCharges ? charges : null,
						amberStars: details?.maxAmberStars ? amberStars : null,
						cyanStars: details?.maxCyanStars ? cyanStars : null,
						subtype: details?.subtypes?.length ? subtype : null,
					},
				});
			}
			item = null;
			onSaved();
		} catch (cause) {
			error = String(cause);
		} finally {
			busy = false;
		}
	}
</script>

{#snippet title()}{dialogTitle}{/snippet}
{#snippet description()}<ListingItemInfo
		name={item?.name ?? ''}
		{mastered}
		ownedCount={item?.quantity ?? 0}
	/>{/snippet}
{#snippet dialogClose()}<Button class={selectedClass('cancel')}>Cancel</Button>{/snippet}
{#snippet dialogActions()}
	{#if !isEditing}
		<Button
			class={selectedClass('hidden')}
			disabled={!valid || busy || loading}
			onclick={() => save(false)}
		>
			Create a hidden listing
		</Button>
	{/if}
	<Button
		class={selectedClass('visible')}
		variant="primary"
		disabled={!valid || busy || (!isEditing && loading)}
		onclick={() => save(true)}
	>
		{busy ? 'Saving...' : isEditing ? 'Save changes' : 'Create listing'}
	</Button>
{/snippet}
{#snippet orderPrices(title: string, orders: OrderPreview[], showQuantity: boolean)}
	<section class="bg-card/50 p-3 border border-border-secondary min-w-0">
		<h3 class="mb-2 font-semibold text-sm">{title}</h3>
		{#each orders as order (order.id)}
			<div class="flex justify-between items-center gap-2 py-0.5 tabular-nums text-sm">
				<span class="flex items-center gap-1">
					{priceFormatter.format(order.platinum)}
					<img src="/icons/platinum.png" alt="platinum" class="size-3.5" />
				</span>
				{#if showQuantity}<span class="text-muted-foreground">
						×{volumeFormatter.format(order.quantity)}
					</span>{/if}
			</div>
		{:else}
			<p class="text-muted-foreground text-sm">No in-game orders</p>
		{/each}
	</section>
{/snippet}
{#snippet overlayControls()}
	<div
		class="flex flex-wrap justify-end items-center self-end gap-x-3 gap-y-1 bg-background px-3 py-2 border border-border-secondary max-w-full text-muted-foreground text-xs"
		aria-label="Listing keyboard shortcuts"
	>
		<span class="flex items-center gap-1">
			<Keybind value={config.hotkeys.cycle} /><Keybind value={config.hotkeys.navigate_right} /> next
		</span>
		<span class="flex items-center gap-1">
			<Keybind value={config.hotkeys.cycle_back} /><Keybind value={config.hotkeys.navigate_left} /> back
		</span>
		<span class="flex items-center gap-1">
			<Keybind value={config.hotkeys.navigate_up} /><Keybind value={config.hotkeys.navigate_down} />
			adjust
		</span>
		<span class="flex items-center gap-1">
			<Keybind value={config.hotkeys.listing_confirm} /> select
		</span>
		<span class="flex items-center gap-1"><Keybind value="Esc" /> close</span>
	</div>
{/snippet}
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
	{dialogActions}
	belowContent={overlayMode ? overlayControls : undefined}
	blurBackdrop={!overlayMode}
	strongBackdrop={overlayMode}
	contentProps={{
		class: `w-[min(42rem,calc(100vw-2rem))] h-auto max-h-[calc(100vh-2rem)] ${overlayMode ? 'overlay-listing-dialog' : ''}`,
	}}
>
	<div class="flex flex-col gap-4 px-6 py-1 overflow-y-auto">
		{#if loading}<p class="text-muted-foreground text-sm">Loading current orders...</p>{:else}
			<div class="bg-card/50 p-3 border border-border-secondary text-center">
				<div
					class="grid grid-cols-3 divide-border-secondary divide-x"
					title="Today's completed trades"
				>
					<div class="px-2">
						<div class="text-muted-foreground text-xs">Median</div>
						<div class="font-semibold tabular-nums text-sm">
							{todayStatistics?.median == null
								? '—'
								: `${priceFormatter.format(todayStatistics.median)}p`}
						</div>
					</div>
					<div class="px-2">
						<div class="text-muted-foreground text-xs">Weighted avg</div>
						<div class="font-semibold tabular-nums text-sm">
							{todayStatistics?.weightedAverage == null
								? '—'
								: `${priceFormatter.format(todayStatistics.weightedAverage)}p`}
						</div>
					</div>
					<div class="px-2">
						<div class="text-muted-foreground text-xs">Volume</div>
						<div class="font-semibold tabular-nums text-sm">
							{todayStatistics?.volume == null
								? '—'
								: volumeFormatter.format(todayStatistics.volume)}
						</div>
					</div>
				</div>
			</div>
			<div class="gap-4 grid grid-cols-1 sm:grid-cols-2">
				{@render orderPrices('Cheapest in-game sell orders', sellOrders, true)}
				{@render orderPrices('Highest in-game buy orders', buyOrders, false)}
			</div>
			{#if !isEditing && (details?.maxRank || details?.maxCharges || details?.subtypes?.length || details?.maxAmberStars || details?.maxCyanStars)}
				<p class="text-muted-foreground text-xs">
					Top order prices may include other ranks or variants. Check the variant fields below
					before posting.
				</p>
			{/if}
		{/if}
		<div class="gap-4 grid grid-cols-2">
			<label class="flex flex-col gap-1 text-sm">
				Price (platinum)
				<input
					type="number"
					min="1"
					max="900000"
					step="1"
					bind:value={price}
					class={`bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground ${selectedClass('price')}`}
				/>
			</label>
			<div class="flex flex-col gap-2">
				<label class="flex flex-col gap-1 text-sm">
					Quantity
					<input
						type="number"
						min="1"
						max="9999"
						step="1"
						bind:value={quantity}
						class={`bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground ${selectedClass('quantity')}`}
					/>
				</label>
				{#if !isEditing || editing?.type === 'sell'}
					<ListingQuantityWarning {quantity} ownedCount={item?.quantity ?? 0} />
				{/if}
			</div>
		</div>
		{#if !isEditing && (details?.maxRank || details?.maxCharges || details?.maxAmberStars || details?.maxCyanStars || details?.subtypes?.length)}
			<div class="gap-3 grid grid-cols-2 pt-3 border-border-secondary border-t">
				{#if details.maxRank}<label class="flex flex-col gap-1 text-sm">
						Rank (0–{details.maxRank})
						<input
							type="number"
							min="0"
							max={details.maxRank}
							step="1"
							bind:value={rank}
							class={`bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground ${selectedClass('rank')}`}
						/>
					</label>{/if}
				{#if details.maxCharges}<label class="flex flex-col gap-1 text-sm">
						Charges (0–{details.maxCharges})
						<input
							type="number"
							min="0"
							max={details.maxCharges}
							step="1"
							bind:value={charges}
							class={`bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground ${selectedClass('charges')}`}
						/>
					</label>{/if}
				{#if details.maxAmberStars}<label class="flex flex-col gap-1 text-sm">
						Amber stars (0–{details.maxAmberStars})
						<input
							type="number"
							min="0"
							max={details.maxAmberStars}
							step="1"
							bind:value={amberStars}
							class={`bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground ${selectedClass('amberStars')}`}
						/>
					</label>{/if}
				{#if details.maxCyanStars}<label class="flex flex-col gap-1 text-sm">
						Cyan stars (0–{details.maxCyanStars})
						<input
							type="number"
							min="0"
							max={details.maxCyanStars}
							step="1"
							bind:value={cyanStars}
							class={`bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground ${selectedClass('cyanStars')}`}
						/>
					</label>{/if}
				{#if details.subtypes?.length}<label class="flex flex-col gap-1 text-sm">
						Subtype
						<select
							bind:value={subtype}
							class={`bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground ${selectedClass('subtype')}`}
						>
							{#each details.subtypes as option}<option value={option}>{option}</option>{/each}
						</select>
					</label>{/if}
			</div>
		{/if}
		{#if error}<p role="alert" class="text-danger text-sm">{error}</p>{/if}
	</div>
</Dialog>

<style>
	:global(.overlay-listing-dialog .overlay-control-selected) {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	:global(
			.overlay-listing-dialog
				:is(input, select, button):focus-visible:not(.overlay-control-selected)
		) {
		outline: none;
	}

	:global(.overlay-listing-dialog :is(input, select):focus-visible) {
		border-color: var(--color-border-secondary);
	}
</style>
