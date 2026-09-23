<script lang="ts">
	import { mastery, setMasteryChecked, dismissMasteryDots, type MasteryItem } from '$lib/mastery.svelte';
	import Select from '$lib/components/Select.svelte';
	import Button from '$lib/components/Button.svelte';
	let { onOpenMarket = () => {} }: { onOpenMarket?: (slug: string) => void } = $props();

	let search = $state('');
	const searchQuery = $derived(search.trim().toLowerCase());
	let category = $state('All');
	let tag = $state('All');
	let progress = $state('All');
	let sort = $state('name');
	let visibleCount = $state(80);
	let expanded = $state<string[]>([]);
	const checked = $derived(new Set(mastery.checked));
	const automatic = $derived(new Set(mastery.automatic));
	const categories = $derived(['All', ...new Set(mastery.items.map((item) => item.category ?? 'Other'))]);
	const tags = $derived(['All', 'Prime', 'Tradeable', 'Has components', ...new Set(mastery.items.flatMap((item) => item.tags ?? []))]);
	const categoryOptions = $derived(categories.map((value) => ({ value, label: value })));
	const tagOptions = $derived(tags.map((value) => ({ value, label: value })));
	const progressOptions = [
		{ value: 'All', label: 'All' },
		{ value: 'Checked', label: 'Checked' },
		{ value: 'Unchecked', label: 'Unchecked' },
	];
	const sortOptions = [
		{ value: 'name', label: 'Name (A–Z)' },
		{ value: 'median-desc', label: 'Median (high to low)' },
		{ value: 'median-asc', label: 'Median (low to high)' },
		{ value: 'ducats-desc', label: 'Ducats (high to low)' },
		{ value: 'ducats-asc', label: 'Ducats (low to high)' },
	];
	const filtered = $derived(mastery.items.filter((item) => {
		if (category !== 'All' && (item.category ?? 'Other') !== category) return false;
		if (tag === 'Prime' && !item.name.includes('Prime')) return false;
		if (tag === 'Tradeable' && !item.marketSlug && !item.components.some((part) => part.tradable)) return false;
		if (tag === 'Has components' && item.components.length === 0) return false;
		if (!['All', 'Prime', 'Tradeable', 'Has components'].includes(tag) && !(item.tags ?? []).includes(tag)) return false;
		if (progress === 'Checked' && !checked.has(item.key)) return false;
		if (progress === 'Unchecked' && checked.has(item.key)) return false;
		return !searchQuery || item.name.toLowerCase().includes(searchQuery);
	}));
	const sorted = $derived.by(() => {
		const items = [...filtered];
		if (sort === 'name') return items.sort((a, b) => a.name.localeCompare(b.name));
		const [field, direction] = sort.split('-');
		return items.sort((a, b) => {
			const first = field === 'median' ? priceFor(a)?.median : ducatsFor(a);
			const second = field === 'median' ? priceFor(b)?.median : ducatsFor(b);
			// Items without a value stay at the end in either direction.
			if (first == null) return second == null ? a.name.localeCompare(b.name) : 1;
			if (second == null) return -1;
			return (direction === 'asc' ? first - second : second - first) || a.name.localeCompare(b.name);
		});
	});
	const visible = $derived(sorted.slice(0, visibleCount));
	const checkedCount = $derived(mastery.items.filter((item) => checked.has(item.key)).length);
	function toggle(key: string) { expanded = expanded.includes(key) ? expanded.filter((entry) => entry !== key) : [...expanded, key]; }
	function openMarket(item: MasteryItem) { if (item.marketSlug) onOpenMarket(item.marketSlug); }
	function wikiUrl(item: MasteryItem, parentName?: string) { return item.wikiaUrl ?? `https://wiki.warframe.com/w/Special:Search?search=${encodeURIComponent(parentName ? `${parentName} ${item.name}` : item.name)}`; }
	function priceFor(item: MasteryItem) { return item.marketSlug ? mastery.prices[item.marketSlug] : undefined; }
	function ducatsFor(item: MasteryItem) { return item.marketSlug ? mastery.ducats[item.marketSlug] ?? item.ducats : item.ducats; }
	function isExpanded(item: MasteryItem) { return expanded.includes(item.key); }
	function updateSearch(event: Event & { currentTarget: HTMLInputElement }) {
		search = event.currentTarget.value;
		visibleCount = 80;
	}
	function resetFilters() {
		search = '';
		category = 'All';
		tag = 'All';
		progress = 'All';
		sort = 'name';
		visibleCount = 80;
	}
</script>

<div class="flex h-full w-full flex-col gap-5 overflow-y-auto bg-background p-6 text-foreground">
	<div class="flex flex-wrap items-start justify-between gap-4">
		<div><h1 class="text-3xl font-bold uppercase tracking-tight">Mastery</h1><p class="text-sm text-muted-foreground">Track gear and components. Use Ctrl+Alt+Home in Warframe to mark recognized items.</p></div>
		<div class="flex items-center gap-3 text-sm"><span>{checkedCount} / {mastery.items.length} checked</span>{#if mastery.automatic.length > 0}<button class="rounded border border-border px-3 py-1.5 hover:bg-muted" onclick={dismissMasteryDots}>Dismiss {mastery.automatic.length} new dots</button>{/if}</div>
	</div>
	{#if mastery.loading}<p class="py-10 text-center text-muted-foreground">Loading mastery items…</p>
	{:else if mastery.error}<p class="rounded border border-destructive p-4 text-destructive">{mastery.error}</p>
	{:else}
		<div class="flex flex-wrap items-end gap-3">
			<div class="min-w-56 flex-1">
				<label for="mastery-search" class="block mb-1.5 font-semibold text-muted-foreground text-xs">Search items</label>
				<input
					id="mastery-search"
					type="search"
					value={search}
					oninput={updateSearch}
					placeholder="Search for an item..."
					class="bg-background p-2 border focus-visible:border-accent outline-none w-full text-foreground placeholder:text-muted-foreground"
				/>
			</div>
			<div class="min-w-36 flex-1"><label for="mastery-category" class="block mb-1.5 font-semibold text-muted-foreground text-xs">Category</label><Select type="single" items={categoryOptions} bind:value={category} triggerProps={{ id: 'mastery-category', class: 'max-w-none h-10' }} /></div>
			<div class="min-w-36 flex-1"><label for="mastery-tag" class="block mb-1.5 font-semibold text-muted-foreground text-xs">Tag</label><Select type="single" items={tagOptions} bind:value={tag} triggerProps={{ id: 'mastery-tag', class: 'max-w-none h-10' }} /></div>
			<div class="min-w-36 flex-1"><label for="mastery-progress" class="block mb-1.5 font-semibold text-muted-foreground text-xs">Progress</label><Select type="single" items={progressOptions} bind:value={progress} triggerProps={{ id: 'mastery-progress', class: 'max-w-none h-10' }} /></div>
			<div class="min-w-40 flex-1"><label for="mastery-sort" class="block mb-1.5 font-semibold text-muted-foreground text-xs">Sort by</label><Select type="single" items={sortOptions} bind:value={sort} triggerProps={{ id: 'mastery-sort', class: 'max-w-none h-10' }} /></div>
			<Button onclick={resetFilters} class="h-10">Reset filters</Button>
		</div>
		<div class="overflow-x-auto rounded-md border border-border bg-card/50"><table class="w-full min-w-[540px] table-fixed text-left text-sm">
			<thead class="bg-muted/60 text-xs uppercase text-muted-foreground"><tr><th class="w-12 px-2 py-3">Check</th><th class="px-2 py-3">Item / components</th><th class="w-20 px-2 py-3">Median</th><th class="w-16 px-2 py-3">Ducats</th><th class="w-18 px-2 py-3">Market</th><th class="w-14 px-2 py-3">Wiki</th></tr></thead>
			<tbody>{#each visible as item (item.key)}
				<tr class="border-t border-border hover:bg-muted/30">
					<td class="px-2 py-2"><input type="checkbox" aria-label={`Check ${item.name}`} checked={checked.has(item.key)} onchange={(event) => setMasteryChecked(item.key, event.currentTarget.checked)} /></td>
					<td class="px-2 py-2 font-medium"><div class="flex items-center gap-1">{#if item.components.length > 0}<button class="w-4 shrink-0 text-muted-foreground" aria-label={`${isExpanded(item) ? 'Collapse' : 'Expand'} ${item.name}`} onclick={() => toggle(item.key)}>{isExpanded(item) ? '▾' : '▸'}</button>{:else}<span class="w-4 shrink-0"></span>{/if}<span class="min-w-0 break-words">{item.name}</span>{#if automatic.has(item.key) || item.components.some((part) => automatic.has(part.key))}<span class="size-2.5 shrink-0 rounded-full bg-accent" title="Automatically added by mastery hotkey" aria-label="Automatically added"></span>{/if}</div><div class="pl-5 text-xs font-normal text-muted-foreground">{item.category ?? item.type ?? 'Other'} · MR {item.masteryReq ?? '—'}{item.tradable || item.marketSlug ? ' · Tradeable' : ''}</div></td>
					<td class="px-2 py-2">{#if priceFor(item)}<span class="inline-flex items-center gap-1" title={priceFor(item)?.from_current_offers ? 'Current offer median; no recent trade median' : 'Recent trade median'}>{priceFor(item)?.median.toLocaleString(undefined, { maximumFractionDigits: 1 })}<img src="/icons/platinum.png" class="size-3.5" alt="platinum" /></span>{:else}—{/if}</td>
					<td class="px-2 py-2">{#if ducatsFor(item) != null}<span class="inline-flex items-center gap-1">{ducatsFor(item)}<img src="/icons/ducats.png" class="size-3.5" alt="ducats" /></span>{:else}—{/if}</td>
					<td class="px-2 py-2">{#if item.marketSlug}<button class="text-primary hover:underline" onclick={() => openMarket(item)}>Market</button>{:else}—{/if}</td>
					<td class="px-2 py-2"><a class="text-primary hover:underline" href={wikiUrl(item)} target="_blank" rel="noopener noreferrer">Wiki ↗</a></td>
				</tr>
				{#if isExpanded(item)}{#each item.components as component, index (component.key)}
					<tr class="border-t border-border/50 bg-muted/10 text-muted-foreground"><td class="px-2 py-1.5"><input type="checkbox" aria-label={`Check ${item.name} ${component.itemCount != null && component.itemCount > 1 ? `${component.itemCount}x ` : ''}${component.name}`} checked={checked.has(component.key)} onchange={(event) => setMasteryChecked(component.key, event.currentTarget.checked)} /></td>
						<td class="px-2 py-1.5"><div class="ml-2 flex items-center gap-1 border-l border-border pl-2"><span>{index === item.components.length - 1 ? '└' : '├'}</span>{#if component.itemCount != null && component.itemCount > 1}<span class="shrink-0 font-semibold text-foreground">{component.itemCount}x</span>{/if}<span class="break-words">{component.name}</span>{#if automatic.has(component.key)}<span class="size-2.5 shrink-0 rounded-full bg-accent" title="Automatically added by mastery hotkey" aria-label="Automatically added"></span>{/if}</div></td>
						<td class="px-2 py-1.5">{#if priceFor(component)}<span class="inline-flex items-center gap-1" title={priceFor(component)?.from_current_offers ? 'Current offer median; no recent trade median' : 'Recent trade median'}>{priceFor(component)?.median.toLocaleString(undefined, { maximumFractionDigits: 1 })}<img src="/icons/platinum.png" class="size-3.5" alt="platinum" /></span>{:else}—{/if}</td>
						<td class="px-2 py-1.5">{#if ducatsFor(component) != null}<span class="inline-flex items-center gap-1">{ducatsFor(component)}<img src="/icons/ducats.png" class="size-3.5" alt="ducats" /></span>{:else}—{/if}</td>
						<td class="px-2 py-1.5">{#if component.marketSlug}<button class="text-primary hover:underline" onclick={() => openMarket(component)}>Market</button>{:else}—{/if}</td><td class="px-2 py-1.5"><a class="text-primary hover:underline" href={wikiUrl(component, item.name)} target="_blank" rel="noopener noreferrer">Wiki ↗</a></td>
					</tr>
				{/each}{/if}
			{:else}<tr><td colspan="6" class="px-4 py-10 text-center text-muted-foreground">No mastery items match these filters.</td></tr>{/each}</tbody>
		</table></div>
		<div class="flex items-center justify-between text-sm text-muted-foreground"><span>Showing {visible.length} of {filtered.length} items</span>{#if visible.length < filtered.length}<button class="rounded border border-border px-3 py-1.5 text-foreground hover:bg-muted" onclick={() => visibleCount += 80}>Show more</button>{/if}</div>
	{/if}
</div>
