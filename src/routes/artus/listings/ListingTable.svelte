<script lang="ts">
	import Table from '$lib/components/Table.svelte';
	import type { TableColumn } from '$lib/components/table-types';
	import type { InventoryItem } from '$lib/inventory';
	import ListingRow from './ListingRow.svelte';
	import type { Listing, ListingItem } from './types';

	let {
		orders,
		itemDetails,
		inventoryItems,
		busy,
		menuOpenFor = $bindable<string | null>(null),
		sortColumn,
		sortDirection,
		onSort,
		onEdit,
		onVisibility,
		onSoldOne,
		onDelete,
		onOpenMarket,
	}: {
		orders: Listing[];
		itemDetails: Record<string, ListingItem>;
		inventoryItems: InventoryItem[];
		busy: boolean;
		menuOpenFor: string | null;
		sortColumn: string;
		sortDirection: 'asc' | 'desc';
		onSort: (key: string) => void;
		onEdit: (order: Listing) => void;
		onVisibility: (order: Listing) => void;
		onSoldOne: (order: Listing, removeInventory: boolean) => void;
		onDelete: (order: Listing) => void;
		onOpenMarket: (slug: string) => void;
	} = $props();

	const columns: TableColumn[] = [
		{ key: 'name', label: 'Name', sortable: true, class: 'min-w-64' },
		{ key: 'type', label: 'Type', class: 'w-24' },
		{ key: 'price', label: 'Price', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'quantity', label: 'Quantity', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'status', label: 'Status', sortable: true, class: 'w-28' },
		{ key: 'actions', label: '', align: 'right', class: 'w-0' },
	];
</script>

{#snippet listingRow(order: Listing)}
	<ListingRow
		{order}
		{itemDetails}
		{inventoryItems}
		{busy}
		bind:menuOpenFor
		{onEdit}
		{onVisibility}
		{onSoldOne}
		{onDelete}
		{onOpenMarket}
	/>
{/snippet}

<Table
	{columns}
	rows={orders}
	rowKey={(order) => order.id}
	renderRow={listingRow}
	{sortColumn}
	{sortDirection}
	{onSort}
	minWidth="900px"
/>
