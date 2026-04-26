<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	type MarketDictionaryItem = {
		name: string;
		slug: string;
	};

	let query = $state('');
	let dictionaryItems = $state<MarketDictionaryItem[]>([]);
	let isLoadingDictionary = $state(true);
	let isSearching = $state(false);
	let statusMessage = $state<string | null>(null);
	let selectedSlug = $state<string | null>(null);
	let resultJson = $state('');

	const normalizedNameLookup = $derived.by(() => {
		const map = new Map<string, MarketDictionaryItem>();
		for (const item of dictionaryItems) {
			map.set(item.name.toLowerCase(), item);
		}
		return map;
	});

	const shownSuggestions = $derived.by(() => {
		const normalizedQuery = query.trim().toLowerCase();
		if (!normalizedQuery) {
			return dictionaryItems.slice(0, 100);
		}

		return dictionaryItems
			.filter((item) => item.name.toLowerCase().includes(normalizedQuery))
			.slice(0, 100);
	});

	async function loadDictionaryItems() {
		isLoadingDictionary = true;
		statusMessage = null;

		try {
			dictionaryItems = await invoke<MarketDictionaryItem[]>('get_market_dictionary_items');
			if (dictionaryItems.length === 0) {
				statusMessage = 'No dictionary items are loaded yet.';
			}
		} catch (error) {
			statusMessage = String(error);
		} finally {
			isLoadingDictionary = false;
		}
	}

	function resolveDictionaryItem(rawQuery: string): MarketDictionaryItem | null {
		const normalizedQuery = rawQuery.trim().toLowerCase();
		if (!normalizedQuery) {
			return null;
		}

		const exactMatch = normalizedNameLookup.get(normalizedQuery);
		if (exactMatch) {
			return exactMatch;
		}

		return (
			dictionaryItems.find((item) => item.name.toLowerCase().startsWith(normalizedQuery)) ??
			dictionaryItems.find((item) => item.name.toLowerCase().includes(normalizedQuery)) ??
			null
		);
	}

	async function submitSearch() {
		if (isSearching || isLoadingDictionary) {
			return;
		}

		const dictionaryItem = resolveDictionaryItem(query);
		if (!dictionaryItem) {
			selectedSlug = null;
			resultJson = '';
			statusMessage = 'No matching dictionary item found.';
			return;
		}

		isSearching = true;
		selectedSlug = dictionaryItem.slug;
		statusMessage = null;

		try {
			const responsePayload = await invoke<unknown>('fetch_market_item_by_slug', {
				slug: dictionaryItem.slug,
			});
			resultJson = JSON.stringify(responsePayload, null, 2);
			statusMessage = `Fetched market payload for ${dictionaryItem.name}.`;
		} catch (error) {
			resultJson = '';
			statusMessage = String(error);
		} finally {
			isSearching = false;
		}
	}

	function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		void submitSearch();
	}

	onMount(() => {
		void loadDictionaryItems();
	});
</script>

<section class="max-w-4xl">
	<div class="flex flex-wrap justify-between items-center gap-2">
		<h2 class="font-semibold text-lg">Market</h2>
		<p class="text-muted-foreground text-sm">
			{isLoadingDictionary
				? 'Loading dictionary...'
				: `${dictionaryItems.length} dictionary item${dictionaryItems.length === 1 ? '' : 's'}`}
		</p>
	</div>

	<form class="flex sm:flex-row flex-col gap-2 mt-4" onsubmit={handleSubmit}>
		<input
			type="search"
			bind:value={query}
			list="market-dictionary-items"
			class="px-2 py-1 border rounded w-full sm:max-w-xl text-sm"
			placeholder="Search item name..."
			aria-label="Search market item"
			disabled={isLoadingDictionary || isSearching}
		/>
		<button
			type="submit"
			class="hover:bg-muted px-3 py-1 border rounded text-sm"
			disabled={isLoadingDictionary || isSearching}
		>
			{isSearching ? 'Searching...' : 'Search'}
		</button>
	</form>

	<datalist id="market-dictionary-items">
		{#each shownSuggestions as item (item.slug)}
			<option value={item.name}></option>
		{/each}
	</datalist>

	{#if selectedSlug}
		<p class="mt-2 text-muted-foreground text-xs">Slug: {selectedSlug}</p>
	{/if}

	{#if statusMessage}
		<p class="mt-2 text-muted-foreground text-sm">{statusMessage}</p>
	{/if}

	{#if resultJson}
		<pre
			class="bg-background mt-4 p-3 border rounded max-h-120 overflow-auto text-xs">{resultJson}</pre>
	{:else}
		<div class="mt-4 p-3 border rounded text-muted-foreground text-sm">
			Press Enter to fetch the selected item from warframe.market.
		</div>
	{/if}
</section>
