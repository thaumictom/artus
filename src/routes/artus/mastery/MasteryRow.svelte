<script lang="ts">
	import Icon from '@iconify/svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import { mastery, setMasteryChecked, type MasteryItem } from '$lib/mastery.svelte';

	let {
		item,
		parentName,
		checked,
		automatic,
		expanded = false,
		onToggle = () => {},
		onOpenMarket = () => {},
	}: {
		item: MasteryItem;
		parentName?: string;
		checked: boolean;
		automatic: boolean;
		expanded?: boolean;
		onToggle?: () => void;
		onOpenMarket?: (slug: string) => void;
	} = $props();

	const isComponent = $derived(parentName !== undefined);
	const hasComponents = $derived(item.components.length > 0);
	const price = $derived(item.marketSlug ? mastery.prices[item.marketSlug] : undefined);
	const ducats = $derived(item.marketSlug ? (mastery.ducats[item.marketSlug] ?? item.ducats) : item.ducats);
	const wikiUrl = $derived(
		item.wikiaUrl ??
		`https://wiki.warframe.com/w/Special:Search?search=${encodeURIComponent(parentName ? `${parentName} ${item.name}` : item.name)}`,
	);

	function toggleFromRow(event: MouseEvent) {
		if (!hasComponents || (event.target as HTMLElement).closest('button, a')) return;
		onToggle();
	}

	function toggleFromKeyboard(event: KeyboardEvent) {
		if (!hasComponents || event.target !== event.currentTarget) return;
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			onToggle();
		}
	}
</script>

<tr
	class={isComponent
		? 'border-t border-border-secondary/50 bg-surface/35 text-muted-foreground transition-colors hover:bg-surface/65'
		: `border-t border-border-secondary transition-colors hover:bg-surface/70 ${hasComponents ? 'cursor-pointer' : ''} ${expanded ? 'bg-surface/55' : ''}`}
	onclick={toggleFromRow}
	onkeydown={toggleFromKeyboard}
	role={hasComponents ? 'button' : undefined}
	tabindex={hasComponents ? 0 : undefined}
	aria-expanded={hasComponents ? expanded : undefined}
>
	<td class="px-4 py-3.5 align-middle">
		<Checkbox
			aria-label={`Check ${parentName ? `${parentName} ` : ''}${item.name}`}
			{checked}
			onCheckedChange={(value) => setMasteryChecked(item.key, value)}
		/>
	</td>
	<td class="px-3 py-3.5 min-w-0 align-middle">
		<div class="flex items-center gap-2.5 min-w-0">
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
					{#if automatic}<span class="bg-accent rounded-full size-2 shrink-0" title="Automatically added by mastery hotkey" aria-label="Automatically added"></span>{/if}
				</div>
				{#if !isComponent}
					<div class="mt-0.5 text-muted-foreground text-xs">
						{item.category ?? item.type ?? 'Other'} · MR {item.masteryReq ?? '—'}{item.tradable || item.marketSlug ? ' · Tradeable' : ''}
					</div>
				{/if}
			</div>
		</div>
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
