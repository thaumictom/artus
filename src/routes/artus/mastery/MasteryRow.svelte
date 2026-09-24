<script lang="ts">
	import Icon from '@iconify/svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import { isOwnedMasteryComponent, mastery, setMasteryChecked, type MasteryItem } from '$lib/mastery.svelte';

	let {
		item,
		parentName,
		ownedCount = 0,
		completedComponents = 0,
		checked,
		automatic,
		expanded = false,
		onToggle = () => {},
		onOpenMarket = () => {},
	}: {
		item: MasteryItem;
		parentName?: string;
		ownedCount?: number;
		completedComponents?: number;
		checked: boolean;
		automatic: boolean;
		expanded?: boolean;
		onToggle?: () => void;
		onOpenMarket?: (slug: string) => void;
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

</script>

{#snippet nameContent()}
	{#if isComponent}
		<span aria-hidden="true" class="ml-2 border-border-secondary border-l border-b w-4 h-4 shrink-0 -translate-y-1"></span>
	{:else if hasComponents}
		<Icon icon="material-symbols:chevron-right-rounded" class={`size-5 text-muted-foreground transition-transform shrink-0 ${expanded ? 'rotate-90' : ''}`} />
	{:else}
		<span class="w-5 shrink-0"></span>
	{/if}
	<div class="min-w-0">
		<div class="flex items-center gap-2 font-semibold text-foreground">
			{#if item.itemCount != null && item.itemCount > 1}<span class="text-accent shrink-0">{item.itemCount}×</span>{/if}
			<span class="break-words">{item.name}</span>
			{#if isComponent && ownedCount > 0}<span class="text-muted-foreground text-xs font-normal whitespace-nowrap shrink-0">{ownedCount} owned</span>{/if}
			{#if automatic}<span class="bg-accent rounded-full size-2 shrink-0" title="Automatically added by mastery hotkey" aria-label="Automatically added"></span>{/if}
		</div>
		{#if !isComponent}
			<div class="mt-0.5 text-muted-foreground text-xs">
				{item.category ?? item.type ?? 'Other'} · MR {item.masteryReq ?? '—'}{item.tradable || item.marketSlug ? ' · Tradeable' : ''}
			</div>
		{/if}
	</div>
{/snippet}

<tr
	class={isComponent
		? 'border-t border-border-secondary/50 bg-surface/35 text-muted-foreground transition-colors hover:bg-surface/65'
		: `border-t border-border-secondary transition-colors hover:bg-surface/70 ${expanded ? 'bg-surface/55' : ''}`}
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
		{#if hasComponents}
			<button type="button" class="flex items-center gap-2.5 min-w-0 w-full text-left cursor-pointer focus-visible:outline-2 focus-visible:outline-accent" aria-label={`Show components of ${item.name}`} aria-expanded={expanded} onclick={onToggle}>
				{@render nameContent()}
			</button>
		{:else}
			<div class="flex items-center gap-2.5 min-w-0">{@render nameContent()}</div>
		{/if}
	</td>
	<td class="px-3 py-3.5 text-right align-middle tabular-nums">
		{#if price}
			<span class="inline-flex items-center justify-end gap-1" title={price.from_current_offers ? 'Current offer median; no recent trade median' : 'Recent trade median'}>
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
		<div class="inline-flex items-center gap-3">
			{#if item.marketSlug}<button class="text-accent hover:underline cursor-pointer" onclick={() => onOpenMarket(item.marketSlug!)}>Market</button>{/if}
			<a class="text-muted-foreground hover:text-foreground hover:underline" href={wikiUrl} target="_blank" rel="noopener noreferrer">Wiki ↗</a>
		</div>
	</td>
</tr>
