<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount, type Snippet } from 'svelte';
	import Button from './Button.svelte';
	import Tooltip from './Tooltip.svelte';
	import { itemMetadataDetails, sanitizeItemDescription } from '$lib/market-catalog';
	import { initializeWarframeItems, itemSubtext, resolveWarframeItem } from '$lib/warframe-item.svelte';
	import { openMarketNotificationTarget } from '$lib/market-navigation.svelte';
	import type { Listing } from '../../routes/artus/listings/types';

	let {
		item, name, mastered, ownedCount, listing, tradable,
		onOpenMarket = (slug: string) => openMarketNotificationTarget(slug, Date.now(), 'sell'),
		showDetails = true, hideOwned = false, hideListing = false, hideMarket = false, showMastered = true, clickable = true,
		hideTooltip = false, nameClass = 'font-semibold', trailing,
	}: {
		item: string;
		name?: string;
		mastered?: boolean;
		ownedCount?: number;
		listing?: Listing | null;
		tradable?: boolean;
		onOpenMarket?: (slug: string) => void;
		showDetails?: boolean;
		hideOwned?: boolean;
		hideListing?: boolean;
		hideMarket?: boolean;
		hideTooltip?: boolean;
		showMastered?: boolean;
		clickable?: boolean;
		nameClass?: string;
		trailing?: Snippet;
	} = $props();
	const resolved = $derived(resolveWarframeItem(item, name));
	// Consumers own the shared context even when rendered outside the main page.
	onMount(initializeWarframeItems);
	// A supplied slug already identifies a market item, including items outside the catalog.
	const slug = $derived(resolved.slug ?? (!item.startsWith('/') && item ? item : undefined));
	const canOpen = $derived(!hideMarket && clickable && !!slug && tradable !== false);
	const description = $derived(resolved.metadata?.description ? sanitizeItemDescription(resolved.metadata.description) : undefined);
	const effects = $derived(resolved.metadata?.levelStats?.at(-1)?.stats.map(sanitizeItemDescription) ?? []);
	const metadataDetails = $derived(itemMetadataDetails(resolved.metadata));
	const subtext = $derived(itemSubtext(resolved.metadata));
	const isMastered = $derived(mastered ?? resolved.mastered);
	const owned = $derived(ownedCount ?? resolved.ownedCount);
	const currentListing = $derived(listing === undefined ? resolved.listing : listing);
	const listingLabel = $derived(currentListing
		? `Listed for ${currentListing.platinum.toLocaleString()} platinum${currentListing.visible ? '' : ' (hidden)'}${currentListing.rank != null ? ` · Rank ${currentListing.rank}` : ''}${currentListing.subtype ? ` · ${currentListing.subtype}` : ''}` : '');
	const reference = $derived(item.startsWith('/') ? item : resolved.gameRef);
</script>

{#snippet itemContent()}
<div class="min-w-0">
	<div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-foreground">
		{#if canOpen}
			<Button variant="link" size="none" class={`inline-flex items-center gap-1 text-foreground text-left ${nameClass}`} onclick={() => onOpenMarket(slug!)}>
				<span class="break-words">{resolved.name}</span>
				<Icon icon="material-symbols:arrow-outward-rounded" class="size-4 shrink-0" />
			</Button>
		{:else}<span class={`break-words ${nameClass}`}>{resolved.name}</span>{/if}
		{#if showMastered && isMastered}
			<span title={hideTooltip ? undefined : 'Mastered'} aria-label="Mastered"><Icon icon="material-symbols:check-circle-rounded" class="size-4 text-accent" /></span>
		{/if}
		{#if !hideOwned && owned > 0}
			<span class="inline-flex items-center gap-1 text-muted-foreground text-xs font-normal whitespace-nowrap" title={hideTooltip ? undefined : `${owned} owned`}>
				<Icon icon="lucide:package" class="size-3.5" />{owned} owned
			</span>
		{/if}
		{#if !hideListing && currentListing}
			<span title={hideTooltip ? undefined : listingLabel} aria-label={listingLabel}><Icon icon="lucide:tag" class={`size-4 ${currentListing.visible ? 'text-accent' : 'text-muted-foreground'}`} /></span>
		{/if}
		{@render trailing?.()}
	</div>
	{#if showDetails && subtext}
		<p class="mt-1 text-muted-foreground text-xs font-normal line-clamp-1">{subtext}</p>
	{/if}
</div>
{/snippet}

{#if hideTooltip}
	{@render itemContent()}
{:else}
	<Tooltip align="start" triggerTag="div" class="block w-full min-w-0 text-left" triggerProps={{ 'aria-label': `Details for ${resolved.name}` }}>
		{#snippet children()}{@render itemContent()}{/snippet}
			{#snippet content()}
				<div class="flex flex-col gap-3 max-h-[min(28rem,60vh)] overflow-y-auto text-xs font-normal">
					<p class="font-semibold text-sm">{resolved.name}</p>
					{#if description}<p class="whitespace-pre-line">{description}</p>{/if}
					{#if effects.length}
						<div>
							<p class="mb-1 font-semibold">Max rank effects</p>
							{#each effects as effect}<p class="whitespace-pre-line">{effect}</p>{/each}
						</div>
					{/if}
					{#if metadataDetails.length}
						<dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
							{#each metadataDetails as detail}
								<dt class="text-muted-foreground">{detail.label}</dt><dd class="text-right break-words">{detail.value}</dd>
							{/each}
						</dl>
					{/if}
					{#if reference}<p class="border-t pt-2 font-mono text-muted-foreground break-all">{reference}</p>{/if}
				</div>
			{/snippet}
</Tooltip>
{/if}
