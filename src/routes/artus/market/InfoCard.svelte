<script lang="ts">
	import type { ItemSchema } from '$lib/schemas';
	import type z from 'zod';
	import type { CatalogItem } from '$lib/market-catalog';
	import Icon from '@iconify/svelte';
	import { Collapsible } from 'bits-ui';
	import { slide } from 'svelte/transition';

	let {
		itemData,
		catalogItem,
		mastered = false,
		ownedCount = 0,
		relatedItems = [],
		onSelectItem,
	}: {
		itemData: z.infer<typeof ItemSchema>;
		catalogItem?: CatalogItem;
		mastered?: boolean;
		ownedCount?: number;
		relatedItems?: { label: string; value: string; itemCount?: number | null; owned: boolean }[];
		onSelectItem?: (slug: string) => void;
	} = $props();
	const number = new Intl.NumberFormat(undefined, { maximumFractionDigits: 2 });
	let details = $derived.by(() => {
		if (!catalogItem) return [];
		const item = catalogItem;
		const rows: { label: string; value: string }[] = [];
		const add = (label: string, value: string | number | null | undefined, suffix = '') => {
			if (value !== undefined && value !== null && value !== '') {
				rows.push({
					label,
					value: `${typeof value === 'number' ? number.format(value) : value}${suffix}`,
				});
			}
		};
		add('Type', item.type);
		add('Rarity', item.rarity);
		add('Mastery rank', item.masteryReq);
		add('Compatible with', item.compatName);
		add('Polarity', item.polarity);
		add('Base drain', item.baseDrain);
		add('Max rank', item.fusionLimit ?? itemData.maxRank);
		add('Health', item.health);
		add('Shields', item.shield);
		add('Armor', item.armor);
		add('Energy', item.power);
		add('Base damage', item.totalDamage);
		add('Critical chance', item.criticalChance == null ? null : item.criticalChance * 100, '%');
		add('Critical multiplier', item.criticalMultiplier, '×');
		add('Status chance', item.procChance == null ? null : item.procChance * 100, '%');
		add('Magazine', item.magazineSize);
		add('Reload', item.reloadTime, ' s');
		add('Build cost', item.buildPrice, ' credits');
		add('Build time', item.buildTime == null ? null : item.buildTime / 3600, ' h');
		add('Released', item.releaseDate);
		return rows;
	});
	let maxRankEffects = $derived(catalogItem?.levelStats?.at(-1)?.stats ?? []);
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
		if (itemData.ducats != null) labels.push(`${itemData.ducats} Ducats`);
		if (ownedCount > 0) labels.push(`${ownedCount} owned`);
		return labels;
	});
</script>

<div class="flex flex-col gap-4 p-4 border w-full max-w-3xl">
	<div class="flex items-center gap-4">
		<img
			src={`https://warframe.market/static/assets/${itemData.i18n?.en.icon}`}
			alt={itemData.i18n?.en.name}
			class="bg-surface h-20 object-contain aspect-square text-transparent"
		/>
		<div class="flex justify-between items-center gap-2 w-full">
			<div class="flex flex-col">
				<div class="flex items-center gap-2">
					<h1 class="font-medium text-lg">{itemData.i18n?.en.name}</h1>
					{#if mastered}
						<span title="Mastered" aria-label="Mastered" class="shrink-0">
							<Icon icon="material-symbols:check-circle-rounded" class="size-4 text-accent" />
						</span>
					{/if}
				</div>
				{#if statusLabels.length}
					<div class="text-muted-foreground text-xs uppercase">
						{statusLabels.join(' · ')}
					</div>
				{/if}
			</div>
			<a
				href={wikiUrl}
				target="_blank"
				rel="noopener noreferrer"
				class="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground text-sm hover:underline"
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
						class="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground text-sm cursor-pointer shrink-0"
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
									? 'px-2 py-1 border border-accent bg-accent/10 text-accent text-xs font-medium'
									: 'px-2 py-1 border text-muted-foreground hover:text-foreground hover:bg-surface text-xs cursor-pointer'}
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
							<p class="text-muted-foreground text-sm whitespace-pre-line">
								{catalogItem.description}
							</p>
						{/if}
						{#if details.length}
							<dl class="gap-x-4 gap-y-3 grid grid-cols-2 sm:grid-cols-3 text-sm">
								{#each details as detail (detail.label)}
									<div>
										<dt class="text-muted-foreground text-xs">{detail.label}</dt>
										<dd>{detail.value}</dd>
									</div>
								{/each}
							</dl>
						{/if}
						{#if maxRankEffects.length}
							<div class="pt-3 border-t text-sm">
								<div class="mb-1 text-muted-foreground text-xs">Max rank effects</div>
								{#each maxRankEffects as effect}
									<p class="whitespace-pre-line">{effect.replaceAll('\\n', '\n')}</p>
								{/each}
							</div>
						{/if}
					</div>
				{/if}
			</Collapsible.Content>
		</Collapsible.Root>
	{/if}
</div>
