<script lang="ts">
	import Combobox from '$lib/components/Combobox.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	type MarketDictionaryItem = {
		label: string;
		value: string;
	};

	type MarketDictionaryEntry = {
		name: string;
		slug: string;
	};

	let dictionaryItems = $state<MarketDictionaryItem[]>([]);
	let isLoadingDictionary = $state(true);
	let isSearching = $state(false);
	let statusMessage = $state<string | null>(null);
	let selectedSlug = $state<string | null>(null);
	let resultJson = $state('');

	async function loadDictionaryItems() {
		isLoadingDictionary = true;
		statusMessage = null;

		try {
			const dictionaryEntries = await invoke<MarketDictionaryEntry[]>(
				'get_market_dictionary_items',
			);
			dictionaryItems = dictionaryEntries.map((entry) => ({
				label: entry.name,
				value: entry.slug,
			}));
			if (dictionaryItems.length === 0) {
				statusMessage = 'No dictionary items are loaded yet.';
			}
		} catch (error) {
			statusMessage = String(error);
		} finally {
			isLoadingDictionary = false;
		}
	}

	async function fetchBySlug(slug: string) {
		if (isSearching || isLoadingDictionary) {
			return;
		}

		const dictionaryItem = dictionaryItems.find((item) => item.value === slug);
		if (!dictionaryItem) {
			selectedSlug = null;
			resultJson = '';
			statusMessage = 'No matching dictionary item found.';
			return;
		}

		isSearching = true;
		selectedSlug = dictionaryItem.value;
		statusMessage = null;

		try {
			const responsePayload = await invoke<unknown>('fetch_market_item_by_slug', {
				slug: dictionaryItem.value,
			});
			resultJson = JSON.stringify(responsePayload, null, 2);
			statusMessage = `Fetched market payload for ${dictionaryItem.label}.`;
		} catch (error) {
			resultJson = '';
			statusMessage = String(error);
		} finally {
			isSearching = false;
		}
	}

	function handleValueChange(nextValue: string) {
		if (!nextValue) {
			selectedSlug = null;
			resultJson = '';
			statusMessage = null;
			return;
		}

		void fetchBySlug(nextValue);
	}

	onMount(() => {
		void loadDictionaryItems();
	});
</script>

<div class="flex mx-auto p-8 w-full max-w-2xl">
	<div class="flex flex-col gap-1 w-full">
		<h1>View prices of any item on warframe.market</h1>
		<Combobox
			onValueChange={handleValueChange}
			type="single"
			items={dictionaryItems}
			disabled={isLoadingDictionary || isSearching}
			inputProps={{ placeholder: 'Search for an item...' }}
		></Combobox>
	</div>
</div>

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
		Select an item to fetch its payload from warframe.market.
	</div>
{/if}
