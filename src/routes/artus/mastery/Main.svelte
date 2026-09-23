<script lang="ts">
	import {
		mastery,
		setMasteryChecked,
		dismissMasteryDots,
		type MasteryItem,
	} from '$lib/mastery.svelte';
	import Select from '$lib/components/Select.svelte';
	import Button from '$lib/components/Button.svelte';
	import { masteryRankProgress, masteryXpFor } from '$lib/mastery-xp';
	let { onOpenMarket = () => {} }: { onOpenMarket?: (slug: string) => void } = $props();

	let search = $state('');
	const searchQuery = $derived(search.trim().toLowerCase());
	let category = $state('All');
	let tag = $state('All');
	let progress = $state('All');
	let sort = $state('name');
	let visibleCount = $state(150);
	let expanded = $state<string[]>([]);
	const checked = $derived(new Set(mastery.checked));
	const automatic = $derived(new Set(mastery.automatic));
	const categories = $derived([
		'All',
		...new Set(mastery.items.map((item) => item.category ?? 'Other')),
	]);
	const tags = $derived([
		'All',
		...new Set([
			'Prime',
			'Tradeable',
			'Has components',
			...mastery.items.flatMap((item) => item.tags ?? []),
		]),
	]);
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
	const filtered = $derived(
		mastery.items.filter((item) => {
			if (category !== 'All' && (item.category ?? 'Other') !== category) return false;
			if (tag === 'Prime' && !item.name.includes('Prime')) return false;
			if (tag === 'Tradeable' && !item.marketSlug && !item.components.some((part) => part.tradable))
				return false;
			if (tag === 'Has components' && item.components.length === 0) return false;
			if (
				!['All', 'Prime', 'Tradeable', 'Has components'].includes(tag) &&
				!(item.tags ?? []).includes(tag)
			)
				return false;
			if (progress === 'Checked' && !checked.has(item.key)) return false;
			if (progress === 'Unchecked' && checked.has(item.key)) return false;
			return !searchQuery || item.name.toLowerCase().includes(searchQuery);
		}),
	);
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
			return (
				(direction === 'asc' ? first - second : second - first) || a.name.localeCompare(b.name)
			);
		});
	});
	const visible = $derived(sorted.slice(0, visibleCount));
	const checkedCount = $derived(mastery.items.filter((item) => checked.has(item.key)).length);
	const earnedXp = $derived(
		mastery.items.reduce(
			(total, item) => total + (checked.has(item.key) ? masteryXpFor(item) : 0),
			0,
		),
	);
	const rankProgress = $derived(masteryRankProgress(earnedXp));
	function toggle(key: string) {
		expanded = expanded.includes(key)
			? expanded.filter((entry) => entry !== key)
			: [...expanded, key];
	}
	function openMarket(item: MasteryItem) {
		if (item.marketSlug) onOpenMarket(item.marketSlug);
	}
	function wikiUrl(item: MasteryItem, parentName?: string) {
		return (
			item.wikiaUrl ??
			`https://wiki.warframe.com/w/Special:Search?search=${encodeURIComponent(parentName ? `${parentName} ${item.name}` : item.name)}`
		);
	}
	function priceFor(item: MasteryItem) {
		return item.marketSlug ? mastery.prices[item.marketSlug] : undefined;
	}
	function ducatsFor(item: MasteryItem) {
		return item.marketSlug ? (mastery.ducats[item.marketSlug] ?? item.ducats) : item.ducats;
	}
	function isExpanded(item: MasteryItem) {
		return expanded.includes(item.key);
	}
	function updateSearch(event: Event & { currentTarget: HTMLInputElement }) {
		search = event.currentTarget.value;
		visibleCount = 150;
	}
	function resetFilters() {
		search = '';
		category = 'All';
		tag = 'All';
		progress = 'All';
		sort = 'name';
		visibleCount = 150;
	}
</script>

<div class="flex flex-col gap-5 bg-background p-6 w-full h-full overflow-y-auto text-foreground">
	<div class="flex flex-wrap justify-between items-start gap-4">
		<div class="flex-1 min-w-56">
			<h1 class="font-bold text-3xl uppercase tracking-tight">Mastery</h1>
			<p class="text-muted-foreground text-sm">
				Track gear and components. Use Ctrl+Alt+Home in Warframe to mark recognized items.
			</p>
		</div>
		<div class="bg-card/50 p-3 border border-border rounded-md w-full sm:w-96 text-sm">
			<div class="flex justify-between items-baseline gap-3">
				<span class="font-semibold">{rankProgress.label} · tracked mastery</span>
				<span class="text-muted-foreground text-xs">
					{checkedCount} / {mastery.items.length} checked
				</span>
			</div>
			<div
				class="bg-muted mt-2 rounded-full h-2 overflow-hidden"
				role="progressbar"
				aria-label="Mastery XP toward next rank"
				aria-valuenow={rankProgress.current}
				aria-valuemin="0"
				aria-valuemax={rankProgress.required}
			>
				<div
					class="bg-accent rounded-full h-full transition-[width]"
					style:width={`${rankProgress.percent}%`}
				></div>
			</div>
			<div class="flex justify-between gap-3 mt-1.5 text-muted-foreground text-xs">
				<span>
					{rankProgress.current.toLocaleString()} / {rankProgress.required.toLocaleString()} XP to next
					rank
				</span>
				<span>{rankProgress.xp.toLocaleString()} total XP</span>
			</div>
			<p class="mt-1 text-muted-foreground text-xs">
				Based on checked gear; missions and Intrinsics are excluded.
			</p>
			{#if mastery.automatic.length > 0}<button
					class="hover:bg-muted mt-2 px-2 py-1 border border-border rounded text-xs"
					onclick={dismissMasteryDots}
				>
					Dismiss {mastery.automatic.length} new dots
				</button>{/if}
		</div>
	</div>
	{#if mastery.loading}<p class="py-10 text-muted-foreground text-center">Loading mastery items…</p>
	{:else if mastery.error}<p class="p-4 border border-destructive rounded text-destructive">
			{mastery.error}
		</p>
	{:else}
		<div class="flex flex-wrap items-end gap-3">
			<div class="flex-1 min-w-56">
				<label
					for="mastery-search"
					class="block mb-1.5 font-semibold text-muted-foreground text-xs"
				>
					Search items
				</label>
				<input
					id="mastery-search"
					type="search"
					value={search}
					oninput={updateSearch}
					placeholder="Search for an item..."
					class="bg-background p-2 border focus-visible:border-accent outline-none w-full text-foreground placeholder:text-muted-foreground"
				/>
			</div>
			<div class="flex-1 min-w-36">
				<label
					for="mastery-category"
					class="block mb-1.5 font-semibold text-muted-foreground text-xs"
				>
					Category
				</label>
				<Select
					type="single"
					items={categoryOptions}
					bind:value={category}
					triggerProps={{ id: 'mastery-category', class: 'max-w-none h-10' }}
				/>
			</div>
			<div class="flex-1 min-w-36">
				<label for="mastery-tag" class="block mb-1.5 font-semibold text-muted-foreground text-xs">
					Tag
				</label>
				<Select
					type="single"
					items={tagOptions}
					bind:value={tag}
					triggerProps={{ id: 'mastery-tag', class: 'max-w-none h-10' }}
				/>
			</div>
			<div class="flex-1 min-w-36">
				<label
					for="mastery-progress"
					class="block mb-1.5 font-semibold text-muted-foreground text-xs"
				>
					Progress
				</label>
				<Select
					type="single"
					items={progressOptions}
					bind:value={progress}
					triggerProps={{ id: 'mastery-progress', class: 'max-w-none h-10' }}
				/>
			</div>
			<div class="flex-1 min-w-40">
				<label for="mastery-sort" class="block mb-1.5 font-semibold text-muted-foreground text-xs">
					Sort by
				</label>
				<Select
					type="single"
					items={sortOptions}
					bind:value={sort}
					triggerProps={{ id: 'mastery-sort', class: 'max-w-none h-10' }}
				/>
			</div>
			<Button onclick={resetFilters} class="h-10">Reset filters</Button>
		</div>
		<div class="bg-card/50 border border-border rounded-md overflow-x-auto">
			<table class="w-full min-w-[540px] text-sm text-left table-fixed">
				<thead class="bg-muted/60 text-muted-foreground text-xs uppercase">
					<tr>
						<th class="px-2 py-3 w-12">Check</th>
						<th class="px-2 py-3">Item / components</th>
						<th class="px-2 py-3 w-20">Median</th>
						<th class="px-2 py-3 w-16">Ducats</th>
						<th class="px-2 py-3 w-18">Market</th>
						<th class="px-2 py-3 w-14">Wiki</th>
					</tr>
				</thead>
				<tbody>
					{#each visible as item (item.key)}
						<tr class="hover:bg-muted/30 border-border border-t">
							<td class="px-2 py-2">
								<input
									type="checkbox"
									aria-label={`Check ${item.name}`}
									checked={checked.has(item.key)}
									onchange={(event) => setMasteryChecked(item.key, event.currentTarget.checked)}
								/>
							</td>
							<td class="px-2 py-2 font-medium">
								<div class="flex items-center gap-1">
									{#if item.components.length > 0}<button
											class="w-4 text-muted-foreground shrink-0"
											aria-label={`${isExpanded(item) ? 'Collapse' : 'Expand'} ${item.name}`}
											onclick={() => toggle(item.key)}
										>
											{isExpanded(item) ? '▾' : '▸'}
										</button>{:else}<span class="w-4 shrink-0"></span>{/if}
									<span class="min-w-0 break-words">{item.name}</span>
									{#if automatic.has(item.key) || item.components.some( (part) => automatic.has(part.key), )}<span
											class="bg-accent rounded-full size-2.5 shrink-0"
											title="Automatically added by mastery hotkey"
											aria-label="Automatically added"
										></span>{/if}
								</div>
								<div class="pl-5 font-normal text-muted-foreground text-xs">
									{item.category ?? item.type ?? 'Other'} · MR {item.masteryReq ??
										'—'}{item.tradable || item.marketSlug ? ' · Tradeable' : ''}
								</div>
							</td>
							<td class="px-2 py-2">
								{#if priceFor(item)}<span
										class="inline-flex items-center gap-1"
										title={priceFor(item)?.from_current_offers
											? 'Current offer median; no recent trade median'
											: 'Recent trade median'}
									>
										{priceFor(item)?.median.toLocaleString(undefined, { maximumFractionDigits: 1 })}
										<img src="/icons/platinum.png" class="size-3.5" alt="platinum" />
									</span>{:else}—{/if}
							</td>
							<td class="px-2 py-2">
								{#if ducatsFor(item) != null}<span class="inline-flex items-center gap-1">
										{ducatsFor(item)}
										<img src="/icons/ducats.png" class="size-3.5" alt="ducats" />
									</span>{:else}—{/if}
							</td>
							<td class="px-2 py-2">
								{#if item.marketSlug}<button
										class="text-primary hover:underline"
										onclick={() => openMarket(item)}
									>
										Market
									</button>{:else}—{/if}
							</td>
							<td class="px-2 py-2">
								<a
									class="text-primary hover:underline"
									href={wikiUrl(item)}
									target="_blank"
									rel="noopener noreferrer"
								>
									Wiki ↗
								</a>
							</td>
						</tr>
						{#if isExpanded(item)}{#each item.components as component, index (component.key)}
								<tr class="bg-muted/10 border-border/50 border-t text-muted-foreground">
									<td class="px-2 py-1.5">
										<input
											type="checkbox"
											aria-label={`Check ${item.name} ${component.itemCount != null && component.itemCount > 1 ? `${component.itemCount}x ` : ''}${component.name}`}
											checked={checked.has(component.key)}
											onchange={(event) =>
												setMasteryChecked(component.key, event.currentTarget.checked)}
										/>
									</td>
									<td class="px-2 py-1.5">
										<div class="flex items-center gap-1 ml-2 pl-2 border-border border-l">
											<span>{index === item.components.length - 1 ? '└' : '├'}</span>
											{#if component.itemCount != null && component.itemCount > 1}<span
													class="font-semibold text-foreground shrink-0"
												>
													{component.itemCount}x
												</span>{/if}
											<span class="break-words">{component.name}</span>
											{#if automatic.has(component.key)}<span
													class="bg-accent rounded-full size-2.5 shrink-0"
													title="Automatically added by mastery hotkey"
													aria-label="Automatically added"
												></span>{/if}
										</div>
									</td>
									<td class="px-2 py-1.5">
										{#if priceFor(component)}<span
												class="inline-flex items-center gap-1"
												title={priceFor(component)?.from_current_offers
													? 'Current offer median; no recent trade median'
													: 'Recent trade median'}
											>
												{priceFor(component)?.median.toLocaleString(undefined, {
													maximumFractionDigits: 1,
												})}
												<img src="/icons/platinum.png" class="size-3.5" alt="platinum" />
											</span>{:else}—{/if}
									</td>
									<td class="px-2 py-1.5">
										{#if ducatsFor(component) != null}<span class="inline-flex items-center gap-1">
												{ducatsFor(component)}
												<img src="/icons/ducats.png" class="size-3.5" alt="ducats" />
											</span>{:else}—{/if}
									</td>
									<td class="px-2 py-1.5">
										{#if component.marketSlug}<button
												class="text-primary hover:underline"
												onclick={() => openMarket(component)}
											>
												Market
											</button>{:else}—{/if}
									</td>
									<td class="px-2 py-1.5">
										<a
											class="text-primary hover:underline"
											href={wikiUrl(component, item.name)}
											target="_blank"
											rel="noopener noreferrer"
										>
											Wiki ↗
										</a>
									</td>
								</tr>
							{/each}{/if}
					{:else}<tr>
							<td colspan="6" class="px-4 py-10 text-muted-foreground text-center">
								No mastery items match these filters.
							</td>
						</tr>{/each}
				</tbody>
			</table>
		</div>
		<div class="flex justify-between items-center text-muted-foreground text-sm">
			<span>Showing {visible.length} of {filtered.length} items</span>
			{#if visible.length < filtered.length}<button
					class="hover:bg-muted px-3 py-1.5 border border-border rounded text-foreground"
					onclick={() => (visibleCount += 80)}
				>
					Show more
				</button>{/if}
		</div>
	{/if}
</div>
