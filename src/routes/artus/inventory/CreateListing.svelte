<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import Dialog from '$lib/components/Dialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import type { InventoryItem } from '$lib/inventory';
	import ListingItemInfo from '$lib/components/ListingItemInfo.svelte';
	import ListingQuantityWarning from '$lib/components/ListingQuantityWarning.svelte';
	type ListingItemDetails = { maxRank?: number; maxCharges?: number; maxAmberStars?: number; maxCyanStars?: number; subtypes?: string[]; bulkTradable?: boolean };

	let { item = $bindable<InventoryItem | null>(null), mastered = false, onCreated = () => {} }: { item: InventoryItem | null; mastered?: boolean; onCreated?: () => void } = $props();
	let open = $derived(item !== null);
	let sell = $state<number[]>([]);
	let buy = $state<number[]>([]);
	let marketMedian = $state<number | null>(null);
	let medianUsesOfferFallback = $state(false);
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
	const valid = $derived(Number.isSafeInteger(price) && price >= 1 && price <= 900000 && Number.isSafeInteger(quantity) && quantity >= 1 && quantity <= 9999 &&
		(!details?.maxRank || (Number.isSafeInteger(rank) && rank >= 0 && rank <= details.maxRank)) &&
		(!details?.maxCharges || (Number.isSafeInteger(charges) && charges >= 0 && charges <= details.maxCharges)) &&
		(!details?.maxAmberStars || (Number.isSafeInteger(amberStars) && amberStars >= 0 && amberStars <= details.maxAmberStars)) &&
		(!details?.maxCyanStars || (Number.isSafeInteger(cyanStars) && cyanStars >= 0 && cyanStars <= details.maxCyanStars)) &&
		(!details?.subtypes?.length || details.subtypes.includes(subtype)));
	$effect(() => {
		if (!item?.slug) return;
		const slug = item.slug;
		quantity = Math.min(Math.max(item.quantity, 1), 9999);
		price = 1; sell = []; buy = []; marketMedian = null; details = null; error = null; loading = true;
		const current = ++requestId;
		void Promise.all([
			invoke<{ data: { sell: { platinum: number }[]; buy: { platinum: number }[] } }>('market_top_orders', { slug }),
			invoke<{ data: ListingItemDetails }>('get_market_item', { slug }),
			invoke<Record<string, { median: number; from_current_offers: boolean }>>('get_mastery_tradeable_prices').catch((): Record<string, { median: number; from_current_offers: boolean }> => ({})),
		])
			.then(([response, itemResponse, prices]) => {
				if (current !== requestId) return;
				sell = response.data.sell.slice(0, 5).map((order) => order.platinum);
				buy = response.data.buy.slice(0, 5).map((order) => order.platinum);
				marketMedian = Number.isFinite(prices[slug]?.median) ? prices[slug].median : null;
				medianUsesOfferFallback = prices[slug]?.from_current_offers ?? false;
				price = sell[0] ?? 1;
				details = itemResponse.data;
				rank = 0; charges = 0; amberStars = 0; cyanStars = 0;
				subtype = itemResponse.data?.subtypes?.[0] ?? '';
			})
			.catch((cause) => { if (current === requestId) error = String(cause); })
			.finally(() => { if (current === requestId) loading = false; });
	});

	async function create(visible: boolean) {
		if (!item?.slug || !valid || busy) return;
		busy = true; error = null;
		try {
			await invoke('market_create_listing', { slug: item.slug, platinum: price, quantity, visible, variant: {
				rank: details?.maxRank ? rank : null,
				charges: details?.maxCharges ? charges : null,
				amberStars: details?.maxAmberStars ? amberStars : null,
				cyanStars: details?.maxCyanStars ? cyanStars : null,
				subtype: details?.subtypes?.length ? subtype : null,
			} });
			item = null;
			onCreated();
		} catch (cause) { error = String(cause); }
		finally { busy = false; }
	}
</script>

{#snippet title()}Create sell listing{/snippet}
{#snippet description()}<ListingItemInfo name={item?.name ?? ''} {mastered} ownedCount={item?.quantity ?? 0} />{/snippet}
{#snippet dialogClose()}<Button>Cancel</Button>{/snippet}
{#snippet dialogActions()}
	<Button disabled={!valid || busy || loading} onclick={() => create(false)}>Create a hidden listing</Button>
	<Button variant="primary" disabled={!valid || busy || loading} onclick={() => create(true)}>{busy ? 'Creating...' : 'Create listing'}</Button>
{/snippet}
<Dialog bind:open={() => open, (value) => { if (!value) item = null; }} {title} {description} {dialogClose} {dialogActions} contentProps={{ class: 'h-auto max-h-[calc(100vh-2rem)]' }}>
	<div class="flex flex-col gap-4 px-6 overflow-y-auto">
		{#if loading}<p class="text-muted-foreground text-sm">Loading current orders...</p>{:else}
			<div class="grid grid-cols-2 gap-4">
				<div class="bg-card/50 p-3 border border-border-secondary">
					<h3 class="mb-2 font-semibold text-sm">Top sell orders</h3>
					{#each sell as value}<div class="py-0.5 tabular-nums text-sm">{value} platinum</div>{:else}<p class="text-muted-foreground text-sm">No sell orders</p>{/each}
					<div class="mt-2 pt-2 border-t border-border-secondary font-semibold text-sm" title={medianUsesOfferFallback ? 'Current offer median; no recent trade median' : 'Recent trade median'}>Market median: {marketMedian === null ? '—' : `${medianUsesOfferFallback ? '~' : ''}${marketMedian} platinum`}</div>
				</div>
				<div class="bg-card/50 p-3 border border-border-secondary">
					<h3 class="mb-2 font-semibold text-sm">Top buy orders</h3>
					{#each buy as value}<div class="py-0.5 tabular-nums text-sm">{value} platinum</div>{:else}<p class="text-muted-foreground text-sm">No buy orders</p>{/each}
				</div>
			</div>
			{#if details?.maxRank || details?.maxCharges || details?.subtypes?.length || details?.maxAmberStars || details?.maxCyanStars}
				<p class="text-muted-foreground text-xs">Top order prices may include other ranks or variants. Check the variant fields below before posting.</p>
			{/if}
		{/if}
		<div class="grid grid-cols-2 gap-4">
			<label class="flex flex-col gap-1 text-sm">Price (platinum)<input type="number" min="1" max="900000" step="1" bind:value={price} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" /></label>
			<label class="flex flex-col gap-1 text-sm">Quantity<input type="number" min="1" max="9999" step="1" bind:value={quantity} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" /></label>
		</div>
		{#if details?.maxRank || details?.maxCharges || details?.maxAmberStars || details?.maxCyanStars || details?.subtypes?.length}
			<div class="grid grid-cols-2 gap-3 border-t border-border-secondary pt-3">
				{#if details.maxRank}<label class="flex flex-col gap-1 text-sm">Rank (0–{details.maxRank})<input type="number" min="0" max={details.maxRank} step="1" bind:value={rank} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" /></label>{/if}
				{#if details.maxCharges}<label class="flex flex-col gap-1 text-sm">Charges (0–{details.maxCharges})<input type="number" min="0" max={details.maxCharges} step="1" bind:value={charges} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" /></label>{/if}
				{#if details.maxAmberStars}<label class="flex flex-col gap-1 text-sm">Amber stars (0–{details.maxAmberStars})<input type="number" min="0" max={details.maxAmberStars} step="1" bind:value={amberStars} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" /></label>{/if}
				{#if details.maxCyanStars}<label class="flex flex-col gap-1 text-sm">Cyan stars (0–{details.maxCyanStars})<input type="number" min="0" max={details.maxCyanStars} step="1" bind:value={cyanStars} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" /></label>{/if}
				{#if details.subtypes?.length}<label class="flex flex-col gap-1 text-sm">Subtype<select bind:value={subtype} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground">{#each details.subtypes as option}<option value={option}>{option}</option>{/each}</select></label>{/if}
			</div>
		{/if}
		<ListingQuantityWarning {quantity} ownedCount={item?.quantity ?? 0} />
		{#if error}<p role="alert" class="text-danger text-sm">{error}</p>{/if}
	</div>
</Dialog>
