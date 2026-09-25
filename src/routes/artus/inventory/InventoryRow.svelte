<script lang="ts">
	import Icon from '@iconify/svelte';
	import { tick } from 'svelte';
	import { inventoryMarketSlug, type InventoryItem } from '$lib/inventory';
	import { marketAccount } from '$lib/market-account.svelte';
	import ActionPopover from '$lib/components/ActionPopover.svelte';

	let {
		item,
		mastered,
		isNew,
		onChangeQuantity,
		onOpenMarket,
		onCreateListing,
	}: {
		item: InventoryItem;
		mastered: boolean;
		isNew: boolean;
		onChangeQuantity: (item: InventoryItem, delta: number) => void;
		onOpenMarket: (slug: string) => void;
		onCreateListing: (item: InventoryItem) => void;
	} = $props();

	const wikiUrl = $derived(
		`https://wiki.warframe.com/w/Special:Search?search=${encodeURIComponent(item.name)}`,
	);
	const marketSlug = $derived(inventoryMarketSlug(item));
	const platinumFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });
	let menuOpen = $state(false);
	async function createListing() {
		if (!marketSlug) return;
		menuOpen = false;
		await tick();
		requestAnimationFrame(() => onCreateListing({ ...item, slug: marketSlug }));
	}
</script>

<tr class="border-t border-border-secondary transition-colors hover:bg-surface/70">
	<td class="px-3 py-3.5 font-semibold text-foreground min-w-48">
		<div class="flex items-center gap-2">
			<span class="break-words">{item.name}</span>
			{#if mastered}
				<span title="Mastered" aria-label="Mastered" class="shrink-0">
					<Icon icon="material-symbols:check-circle-rounded" class="size-4 text-accent" />
				</span>
			{/if}
			{#if isNew}
				<span
					class="bg-accent rounded-full size-2 shrink-0"
					title="Newly added to inventory"
					aria-label="Newly added"
				></span>
			{/if}
		</div>
		{#if item.category}<div class="mt-0.5 text-muted-foreground text-xs font-normal">{item.category}</div>{/if}
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums">
		<div class="inline-flex items-center gap-2">
			<button
				class="flex items-center justify-center border border-border-secondary size-6 hover:bg-surface cursor-pointer"
				aria-label={`Remove one ${item.name}`}
				onclick={() => onChangeQuantity(item, -1)}
			><Icon icon="lucide:minus" class="size-3.5" /></button>
			<span class="min-w-6 text-center">{item.quantity}</span>
			<button
				class="flex items-center justify-center border border-border-secondary size-6 hover:bg-surface cursor-pointer"
				aria-label={`Add one ${item.name}`}
				onclick={() => onChangeQuantity(item, 1)}
			><Icon icon="lucide:plus" class="size-3.5" /></button>
		</div>
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums">
		{#if item.marketMedian != null}
			<span class="inline-flex items-center justify-end gap-1" title={item.marketMedianUsesOfferFallback ? 'Current offer median; no recent trade median' : 'Recent trade median'}>
				{platinumFormatter.format(item.marketMedian)}
				<img src="/icons/platinum.png" class="size-3.5" alt="platinum" />
			</span>
		{:else}<span class="text-muted-foreground">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums">
		{#if item.ducats != null}
			<span class="inline-flex items-center justify-end gap-1">{item.ducats.toLocaleString()}<img src="/icons/ducats.png" class="size-3.5" alt="ducats" /></span>
		{:else}<span class="text-muted-foreground">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums font-semibold">
		{#if item.marketMedian != null}
			<span class="inline-flex items-center justify-end gap-1">{platinumFormatter.format(item.marketMedian * item.quantity)}<img src="/icons/platinum.png" class="size-3.5" alt="platinum" /></span>
		{:else}<span class="text-muted-foreground font-normal">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 text-right tabular-nums font-semibold">
		{#if item.ducats != null}
			<span class="inline-flex items-center justify-end gap-1">{(item.ducats * item.quantity).toLocaleString()}<img src="/icons/ducats.png" class="size-3.5" alt="ducats" /></span>
		{:else}<span class="text-muted-foreground font-normal">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 text-right">
		<ActionPopover bind:open={menuOpen} triggerAriaLabel={`Actions for ${item.name}`} triggerClass="inline-flex items-center justify-center hover:bg-elevated border border-border-secondary size-7" contentClass="w-44">
			{#snippet trigger()}<Icon icon="lucide:ellipsis" class="size-4" />{/snippet}
			{#if marketSlug}<button type="button" onclick={() => { menuOpen = false; onOpenMarket(marketSlug!); }} class="flex items-center gap-2 hover:bg-elevated px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:store" class="size-4" /> View market</button>{/if}
			{#if marketSlug}<button type="button" disabled={!marketAccount.session} title={marketAccount.session ? undefined : 'Log in to warframe.market to create a listing'} onclick={createListing} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-40 disabled:cursor-not-allowed px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:plus" class="size-4" /> Create listing</button>{/if}
			<a href={wikiUrl} target="_blank" rel="noopener noreferrer" onclick={() => (menuOpen = false)} class="flex items-center gap-2 hover:bg-elevated px-2 py-1.5 text-sm"><Icon icon="lucide:external-link" class="size-4" /> View wiki</a>
		</ActionPopover>
	</td>
</tr>
