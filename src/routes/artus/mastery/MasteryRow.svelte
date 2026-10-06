<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import WarframeItem from '$lib/components/WarframeItem.svelte';
	import { isOwnedMasteryComponent, mastery, setMasteryChecked, type MasteryItem } from '$lib/mastery.svelte';

	let {
		item,
		virtualIndex,
		measureRow,
		parentName,
		ownedCount = 0,
		completedComponents = 0,
		checked,
		automatic,
		expanded = false,
		onToggle = () => {},
		onOpenMarket = () => {},
		onBuy = () => {},
	}: {
		item: MasteryItem;
		virtualIndex: number;
		measureRow: (element: HTMLTableRowElement) => void;
		parentName?: string;
		ownedCount?: number;
		completedComponents?: number;
		checked: boolean;
		automatic: boolean;
		expanded?: boolean;
		onToggle?: () => void;
		onOpenMarket?: (slug: string) => void;
		onBuy?: (slug: string, name: string) => void;
	} = $props();

	const isComponent = $derived(parentName !== undefined);
	const hasComponents = $derived(item.components.length > 0);
	const allComponentsComplete = $derived(hasComponents && completedComponents === item.components.length);
	const ownedOnly = $derived(!checked && (allComponentsComplete || (isComponent && isOwnedMasteryComponent(item, ownedCount))));
	const showProgress = $derived(hasComponents && !checked && completedComponents > 0);
	const progressBars = $derived(Math.min(item.components.length, 5));
	const filledBars = $derived.by(() => {
		if (item.components.length <= 5) return completedComponents;
		if (completedComponents === 0) return 0;
		if (completedComponents === item.components.length) return progressBars;
		return Math.max(1, Math.min(progressBars - 1, Math.round(completedComponents / item.components.length * progressBars)));
	});
	const price = $derived(item.marketSlug ? mastery.prices[item.marketSlug] : undefined);
	const ducats = $derived(item.marketSlug ? (mastery.ducats[item.marketSlug] ?? item.ducats) : item.ducats);
	const wikiUrl = $derived(
		item.wikiaUrl ??
		`https://wiki.warframe.com/w/Special:Search?search=${encodeURIComponent(parentName ? `${parentName} ${item.name}` : item.name)}`,
	);
	function buy() {
		if (!item.marketSlug) return;
		onBuy(item.marketSlug, parentName && !item.name.startsWith(parentName) ? `${parentName} ${item.name}` : item.name);
	}
	function handleRowClick(event: MouseEvent) {
		if (hasComponents && !(event.target as HTMLElement).closest('button, a, [role="checkbox"]')) onToggle();
	}
	function handleRowKeydown(event: KeyboardEvent) {
		if (hasComponents && event.target === event.currentTarget && (event.key === 'Enter' || event.key === ' ')) {
			event.preventDefault();
			onToggle();
		}
	}

</script>

{#snippet nameContent()}
	<div class="min-w-0">
		<div class="flex items-center gap-2 font-semibold text-foreground">
			{#if item.itemCount != null && item.itemCount > 1}<span class="text-accent shrink-0">{item.itemCount}×</span>{/if}
			<WarframeItem item={item.marketSlug ?? item.key} name={item.name} {ownedCount}
				mastered={checked} {onOpenMarket} showMastered={!isComponent}>
				{#snippet trailing()}
					{#if automatic}<span class="bg-accent rounded-full size-2 shrink-0" title="Automatically added by mastery hotkey" aria-label="Automatically added"></span>{/if}
				{/snippet}
			</WarframeItem>
		</div>
	</div>
{/snippet}

<tr
	data-index={virtualIndex}
	use:measureRow
	class={isComponent
		? `border-t border-border-secondary/50 bg-surface/35 text-muted-foreground transition-colors hover:bg-surface/65 ${hasComponents ? 'cursor-pointer focus-visible:outline-2 focus-visible:outline-accent' : ''}`
		: `border-t border-border-secondary transition-colors hover:bg-surface/70 ${expanded ? 'bg-surface/55' : ''} ${hasComponents ? 'cursor-pointer focus-visible:outline-2 focus-visible:outline-accent' : ''}`}
	onclick={handleRowClick}
	onkeydown={handleRowKeydown}
	tabindex={hasComponents ? 0 : undefined}
	role={hasComponents ? 'button' : undefined}
	aria-label={hasComponents ? `${expanded ? 'Hide' : 'Show'} components of ${item.name}` : undefined}
	aria-expanded={hasComponents ? expanded : undefined}
>
	<td class="px-4 py-3.5 align-middle">
		<div class="flex items-center gap-1">
			<Checkbox
				aria-label={ownedOnly ? `Mark ${parentName ? `${parentName} ` : ''}${item.name} as mastered` : `Check ${parentName ? `${parentName} ` : ''}${item.name}`}
				{checked}
				owned={ownedOnly}
				indeterminate={showProgress && !allComponentsComplete}
				onCheckedChange={(value) => setMasteryChecked(item.key, value)}
			/>
			{#if showProgress}
				<span
					class="flex flex-col gap-0.5 shrink-0"
					role="img"
					aria-label={`${completedComponents} of ${item.components.length} components checked or owned`}
					title={`${completedComponents} of ${item.components.length} components checked or owned`}
				>
					{#each Array.from({ length: progressBars }) as _, index}
						<span class={`w-1.5 h-0.5 ${index < filledBars ? 'bg-accent' : 'bg-muted-foreground/50'}`}></span>
					{/each}
				</span>
			{/if}
		</div>
	</td>
	<td class="px-3 py-3.5 min-w-0 align-middle">
		<div class="flex items-center gap-2.5 min-w-0">
			{#if isComponent}
				<span aria-hidden="true" class="ml-2 border-border-secondary border-l border-b w-4 h-4 shrink-0 -translate-y-1"></span>
			{:else if hasComponents}
				<button type="button" class="shrink-0 cursor-pointer focus-visible:outline-2 focus-visible:outline-accent" aria-label={`${expanded ? 'Hide' : 'Show'} components of ${item.name}`} aria-expanded={expanded} onclick={onToggle}>
					<Icon icon="material-symbols:chevron-right-rounded" class={`size-5 text-muted-foreground transition-transform ${expanded ? 'rotate-90' : ''}`} />
				</button>
			{:else}
				<span class="w-5 shrink-0"></span>
			{/if}
			{@render nameContent()}
		</div>
	</td>
	<td class="px-3 py-3.5 text-right align-middle tabular-nums">
		{#if price}
			<span class="inline-flex items-center justify-end gap-1" title={price.from_current_offers ? 'Current offer median; recent trades exist' : 'Recent trade median'}>
				{price.median.toLocaleString(undefined, { maximumFractionDigits: 1 })}
				<img src="/icons/platinum.png" class="size-3.5" alt="platinum" />
			</span>
		{:else}<span class="text-muted-foreground">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 text-right align-middle tabular-nums">
		{#if ducats != null}
			<span class="inline-flex items-center justify-end gap-1">
				{ducats.toLocaleString()}
				<img src="/icons/ducats.png" class="size-3.5" alt="ducats" />
			</span>
		{:else}<span class="text-muted-foreground">—</span>{/if}
	</td>
	<td class="px-3 py-3.5 text-right align-middle">
		<div class="flex justify-end items-center gap-1.5">
			{#if item.marketSlug}
				<Button size="icon" class="inline-flex justify-center items-center size-8" title="Buy" aria-label={`Buy ${parentName ? `${parentName} ` : ''}${item.name}`} onclick={buy}>
					<Icon icon="lucide:shopping-cart" class="size-4" />
				</Button>
			{/if}
			<Button size="icon" class="inline-flex justify-center items-center size-8" href={wikiUrl} target="_blank" rel="noopener noreferrer" title="View wiki" aria-label={`View ${parentName ? `${parentName} ` : ''}${item.name} on the wiki`}>
				<Icon icon="lucide:book-open" class="size-4" />
			</Button>
		</div>
	</td>
</tr>
