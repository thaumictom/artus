<script lang="ts">
	import { onDestroy, tick } from 'svelte';
	import Button from '$lib/components/Button.svelte';
	import Combobox from '$lib/components/Combobox.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import type { InventoryItem } from '$lib/inventory';
	import { ownedMarketCount } from '$lib/listing-context';
	import { warframeItems, refreshWarframeItemCatalog } from '$lib/warframe-item.svelte';

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
	const catalogItems = $derived(warframeItems.addOptions);
	const loading = $derived(!warframeItems.identitiesReady);
	const error = $derived(warframeItems.identitiesError);
	let disposed = false;
	let selectionFrame = 0;
	onDestroy(() => {
		disposed = true;
		cancelAnimationFrame(selectionFrame);
	});
	let selectedItem = $derived(catalogItems.find((item) => item.value === selectedSlug));

	async function selectItem() {
		if (!selectedItem) return;
		const selected = selectedItem;
		open = false;
		selectedSlug = '';
		await tick();
		if (disposed) return;
		cancelAnimationFrame(selectionFrame);
		selectionFrame = requestAnimationFrame(() => onSelect({
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
		<Combobox type="single" items={catalogItems} bind:value={selectedSlug} inputValue={selectedItem?.label ?? ''} disabled={loading || !!error} inputProps={{ id: 'listing-create-item', placeholder: 'Search for an item...' }} />
		{#if error}<p role="alert" class="mt-2 text-danger text-base">Could not load items. <button type="button" class="underline cursor-pointer" onclick={refreshWarframeItemCatalog}>Retry</button></p>{/if}
	</div>
</Dialog>
