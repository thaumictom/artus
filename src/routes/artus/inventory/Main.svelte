<script lang="ts">
	import * as Table from '$lib/components/ui/table/index.js';

	type InventoryItem = {
		name: string;
		quantity: number;
		marketMedian?: number;
		marketMedianUsesOfferFallback?: boolean;
	};

	let searchQuery = $state('');

	let {
		items,
		onIncrease,
		onDecrease,
		clearInventory,
	}: {
		items: InventoryItem[];
		onIncrease: (name: string) => void;
		onDecrease: (name: string) => void;
		clearInventory: () => void;
	} = $props();

	const totalQuantity = $derived.by(() => items.reduce((sum, item) => sum + item.quantity, 0));
	const normalizedSearchQuery = $derived.by(() => searchQuery.trim().toLowerCase());
	const filteredItems = $derived.by(() => {
		if (!normalizedSearchQuery) {
			return items;
		}

		return items.filter((item) => item.name.toLowerCase().includes(normalizedSearchQuery));
	});
	const hasOfferFallbackMedian = $derived.by(() =>
		items.some((item) => item.marketMedianUsesOfferFallback === true),
	);
	const marketMedianFormatter = new Intl.NumberFormat(undefined, {
		minimumFractionDigits: 0,
		maximumFractionDigits: 2,
	});

	function formatMarketMedian(item: InventoryItem): string {
		if (typeof item.marketMedian !== 'number' || !Number.isFinite(item.marketMedian)) {
			return '—';
		}

		const formatted = marketMedianFormatter.format(item.marketMedian);
		return item.marketMedianUsesOfferFallback ? `${formatted}*` : formatted;
	}
</script>

<section class="max-w-5xl">
	<div class="flex flex-wrap justify-between items-center gap-2">
		<h2 class="font-semibold text-lg">Inventory</h2>
		<p class="text-muted-foreground text-sm">
			{items.length} unique item{items.length === 1 ? '' : 's'} • {totalQuantity} total
		</p>
	</div>

	{#if items.length === 0}
		<div class="mt-4 p-4 border rounded">
			<p class="text-sm">
				Press the hotkey to detect items. Each detected item is added here automatically.
			</p>
		</div>
	{:else}
		<div class="flex flex-wrap justify-between items-center gap-2 mt-4">
			<div class="flex flex-wrap sm:flex-1 items-center gap-2 w-full sm:w-auto">
				<input
					type="search"
					bind:value={searchQuery}
					class="px-2 py-1 border rounded w-full sm:max-w-sm text-sm"
					placeholder="Search inventory..."
					aria-label="Search inventory"
				/>
				<p class="text-muted-foreground text-xs whitespace-nowrap">
					{filteredItems.length} shown of {items.length}
				</p>
			</div>
			<button
				type="button"
				class="hover:bg-muted px-3 py-1 border rounded text-sm"
				onclick={clearInventory}
			>
				Clear inventory
			</button>
		</div>

		<div class="mt-3 border rounded-md max-h-112 overflow-y-auto">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head class="top-0 sticky bg-background">Item</Table.Head>
						<Table.Head class="top-0 sticky bg-background w-24 text-right">Quantity</Table.Head>
						<Table.Head class="top-0 sticky bg-background w-24 text-right">Median</Table.Head>
						<Table.Head class="top-0 sticky bg-background w-28 text-right">Adjust</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each filteredItems as item (item.name)}
						<Table.Row>
							<Table.Cell class="font-medium">{item.name}</Table.Cell>
							<Table.Cell class="text-right">{item.quantity}</Table.Cell>
							<Table.Cell class="text-right">{formatMarketMedian(item)}</Table.Cell>
							<Table.Cell>
								<div class="flex justify-end items-center gap-1">
									<button
										type="button"
										class="hover:bg-muted border rounded w-8 h-8 font-semibold text-sm leading-none"
										onclick={() => onDecrease(item.name)}
										aria-label={`Decrease quantity for ${item.name}`}
									>
										-
									</button>
									<button
										type="button"
										class="hover:bg-muted border rounded w-8 h-8 font-semibold text-sm leading-none"
										onclick={() => onIncrease(item.name)}
										aria-label={`Increase quantity for ${item.name}`}
									>
										+
									</button>
								</div>
							</Table.Cell>
						</Table.Row>
					{:else}
						<Table.Row>
							<Table.Cell colspan={4} class="h-20 text-muted-foreground text-center">
								No items match your search.
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</div>

		{#if hasOfferFallbackMedian}
			<p class="mt-2 text-muted-foreground text-xs">* median from current offers</p>
		{/if}
	{/if}
</section>
