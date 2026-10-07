<script lang="ts">
	import Currency from '$lib/components/Currency.svelte';
	import type { ItemSchema } from '$lib/schemas';
	import type z from 'zod';
	import { itemMetadataDetails, sanitizeItemDescription, type CatalogItem } from '$lib/market-catalog';
	import Icon from '@iconify/svelte';
	import { Collapsible } from 'bits-ui';
	import { slide } from 'svelte/transition';
	import WarframeItem from '$lib/components/WarframeItem.svelte';

	let {
		itemData,
		catalogItem,
		relatedItems = [],
		onSelectItem,
	}: {
		itemData: z.infer<typeof ItemSchema>;
		catalogItem?: CatalogItem;
		relatedItems?: { label: string; value: string; itemCount?: number | null; owned: boolean }[];
		onSelectItem?: (slug: string) => void;
	} = $props();
	let details = $derived(itemMetadataDetails(catalogItem, itemData.maxRank));
	let maxRankEffects = $derived(catalogItem?.levelStats?.at(-1)?.stats.map(sanitizeItemDescription) ?? []);
	let hasMoreInfo = $derived(
		Boolean(catalogItem?.description || details.length || maxRankEffects.length),
	);
	let moreInfoOpen = $state(false);
	$effect(() => {
		itemData.slug;
		moreInfoOpen = false;
	});
	let wikiUrl = $derived(
		catalogItem?.wikiaUrl ||
			itemData.i18n?.en?.wikiLink ||
			`https://wiki.warframe.com/w/Special:Search?search=${encodeURIComponent(itemData.i18n?.en?.name ?? itemData.slug)}`,
	);
	let statusLabels = $derived.by(() => {
		const labels: string[] = [];
		if (itemData.vaulted) labels.push('Vaulted');
		return labels;
	});
</script>

<div class="page-width flex flex-col gap-4 p-4 border w-full max-w-3xl">
	<div class="flex items-center gap-4">
		<img
			src={`https://warframe.market/static/assets/${itemData.i18n?.en.icon}`}
			alt={itemData.i18n?.en.name}
			class="bg-surface h-20 object-contain aspect-square text-transparent"
		/>
		<div class="flex justify-between items-center gap-2 w-full">
			<div class="flex flex-col">
				<h1 class="sr-only">{itemData.i18n?.en.name}</h1>
				<WarframeItem item={itemData.slug} name={itemData.i18n?.en.name}
					clickable={false} nameClass="font-medium text-xl" />
				{#if statusLabels.length || itemData.ducats != null}
					<div class="flex items-center gap-1 text-muted-foreground text-sm uppercase">
						{statusLabels.join(' · ')}
						{#if statusLabels.length && itemData.ducats != null}<span aria-hidden="true">·</span>{/if}
						{#if itemData.ducats != null}<Currency value={itemData.ducats} currency="ducats" />{/if}
					</div>
				{/if}
			</div>
			<a
				href={wikiUrl}
				target="_blank"
				rel="noopener noreferrer"
				class="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground text-base hover:underline"
			>
				Wiki
				<Icon icon="material-symbols:arrow-outward-rounded" class="size-4" />
			</a>
		</div>
	</div>
	{#if hasMoreInfo || relatedItems.length > 1}
		<Collapsible.Root bind:open={moreInfoOpen}>
			<div class="flex items-center gap-3">
				{#if hasMoreInfo}
					<Collapsible.Trigger
						class="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground text-base cursor-pointer shrink-0"
					>
						More info
						<Icon
							icon="material-symbols:expand-more-rounded"
							class={moreInfoOpen ? 'size-4 rotate-180' : 'size-4'}
						/>
					</Collapsible.Trigger>
				{/if}
				<div class="flex-1 bg-surface min-w-4 h-px" aria-hidden="true"></div>
				{#if relatedItems.length > 1}
					<nav
						aria-label="Set and tradable components"
						class="flex flex-wrap justify-end gap-2 min-w-0"
					>
						{#each relatedItems as item (item.value)}
							<button
								type="button"
								onclick={() => onSelectItem?.(item.value)}
								aria-current={item.value === itemData.slug ? 'page' : undefined}
								class={item.value === itemData.slug
									? 'px-2 py-1 border border-accent bg-accent/10 text-accent text-sm font-medium'
									: 'px-2 py-1 border text-muted-foreground hover:text-foreground hover:bg-surface text-sm cursor-pointer'}
							>
								<span class="inline-flex items-center gap-1">
									<span>
										{#if item.itemCount != null && item.itemCount > 1}{item.itemCount}x&nbsp;{/if}{item.label}
									</span>
									{#if item.owned}<span
											class="inline-flex justify-center items-center border-current size-3.5"
											aria-label="Owned"
										>
											<Icon icon="material-symbols:check-rounded" class="size-4" />
										</span>{/if}
								</span>
							</button>
						{/each}
					</nav>
				{/if}
			</div>
			<Collapsible.Content forceMount>
				{#if moreInfoOpen && hasMoreInfo}
					<div transition:slide class="flex flex-col gap-4 pt-4">
						{#if catalogItem?.description}
							<p class="text-muted-foreground text-base whitespace-pre-line">
								{sanitizeItemDescription(catalogItem.description)}
							</p>
						{/if}
						{#if details.length}
							<dl class="gap-x-4 gap-y-3 grid grid-cols-2 sm:grid-cols-3 text-base">
								{#each details as detail (detail.label)}
									<div>
										<dt class="text-muted-foreground text-sm">{detail.label}</dt>
										<dd>
											{#if detail.currency}<Currency value={detail.value} currency={detail.currency} />
											{:else}{detail.value}{/if}
										</dd>
									</div>
								{/each}
							</dl>
						{/if}
						{#if maxRankEffects.length}
							<div class="pt-3 border-t text-base">
								<div class="mb-1 text-muted-foreground text-sm">Max rank effects</div>
								{#each maxRankEffects as effect}
									<p class="whitespace-pre-line">{effect}</p>
								{/each}
							</div>
						{/if}
					</div>
				{/if}
			</Collapsible.Content>
		</Collapsible.Root>
	{/if}
</div>
