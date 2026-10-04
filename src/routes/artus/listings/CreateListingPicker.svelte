<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { tick } from 'svelte';
	import Button from '$lib/components/Button.svelte';
	import Combobox from '$lib/components/Combobox.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import type { InventoryItem } from '$lib/inventory';
	import { ownedMarketCount } from '$lib/listing-context';
	import { DictionarySchema } from '$lib/schemas';

	let {
		open = $bindable(false),
		inventoryItems,
		onSelect,
	}: {
		open: boolean;
		inventoryItems: InventoryItem[];
		onSelect: (item: InventoryItem) => void;
	} = $props();
	let selectedSlug = $state('');
	let catalogItems = $state<{ label: string; value: string }[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let selectedItem = $derived(catalogItems.find((item) => item.value === selectedSlug));

	$effect(() => {
		if (open && catalogItems.length === 0 && !loading && !error) void loadCatalog();
	});

	async function loadCatalog() {
		loading = true;
		error = null;
		try {
			const dictionary = DictionarySchema.parse(await invoke('get_market_dictionary'));
			catalogItems = dictionary.items.map((item) => ({ label: item.name, value: item.slug }));
		} catch (cause) {
			error = String(cause);
		} finally {
			loading = false;
		}
	}

	async function selectItem() {
		if (!selectedItem) return;
		const selected = selectedItem;
		open = false;
		selectedSlug = '';
		await tick();
		requestAnimationFrame(() => onSelect({
			name: selected.label,
			slug: selected.value,
			quantity: ownedMarketCount(inventoryItems, selected.value, selected.label),
		}));
	}
</script>

{#snippet title()}Create sell listing{/snippet}
{#snippet description()}Search the market item list to create a listing. You can list items you do not own.{/snippet}
{#snippet close()}<Button>Cancel</Button>{/snippet}
{#snippet actions()}<Button variant="primary" disabled={!selectedItem} onclick={selectItem}>Continue</Button>{/snippet}
<Dialog bind:open {title} {description} dialogClose={close} dialogActions={actions} contentProps={{ class: 'h-auto' }}>
	<div class="px-6">
		<label for="listing-create-item" class="block mb-1.5 font-semibold text-muted-foreground text-sm">Item</label>
		{#if loading && catalogItems.length === 0}
			<div role="status" aria-label="Loading market items"><Skeleton class="w-full h-10" /></div>
		{:else}
			<Combobox type="single" items={catalogItems} bind:value={selectedSlug} inputValue={selectedItem?.label ?? ''} disabled={!!error} inputProps={{ id: 'listing-create-item', placeholder: 'Search for an item...' }} />
		{/if}
		{#if error}<p role="alert" class="mt-2 text-danger text-base">Could not load items. <button type="button" class="underline cursor-pointer" onclick={loadCatalog}>Retry</button></p>{/if}
	</div>
</Dialog>
