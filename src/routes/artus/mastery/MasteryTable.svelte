<script lang="ts">
	import Table from '$lib/components/Table.svelte';
	import type { TableColumn } from '$lib/components/table-types';
	import type { MasteryItem } from '$lib/mastery.svelte';
	import MasteryRow from './MasteryRow.svelte';

	type SortColumn = 'name' | 'median' | 'ducats';
	let {
		items,
		checked,
		automatic,
		expanded,
		sortColumn,
		sortDirection,
		onSort,
		onToggle,
		onOpenMarket,
		onBuy,
		ownedComponentCount,
		completedComponentCount,
	}: {
		items: MasteryItem[];
		checked: Set<string>;
		automatic: Set<string>;
		expanded: string[];
		sortColumn: SortColumn;
		sortDirection: 'asc' | 'desc';
		onSort: (column: SortColumn) => void;
		onToggle: (key: string) => void;
		onOpenMarket: (slug: string) => void;
		onBuy: (slug: string, name: string) => void;
		ownedComponentCount: (component: MasteryItem, parentName: string) => number;
		completedComponentCount: (item: MasteryItem) => number;
	} = $props();

	const columns: TableColumn[] = [
		{ key: 'checked', label: '', class: 'w-18 px-4' },
		{ key: 'name', label: 'Item', sortable: true },
		{ key: 'median', label: 'Median', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'ducats', label: 'Ducats', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'actions', label: '', align: 'right', class: 'w-0' },
	];
	type MasteryTableRow = { item: MasteryItem; parentName?: string };
	const rows = $derived(
		items.flatMap((item): MasteryTableRow[] => [
			{ item },
			...(expanded.includes(item.key)
				? item.components.map((component) => ({ item: component, parentName: item.name }))
				: []),
		]),
	);
</script>

{#snippet masteryRow({ item, parentName }: MasteryTableRow, index: number, measureRow: (element: HTMLTableRowElement) => void)}
	<MasteryRow
		virtualIndex={index}
		{measureRow}
		{item}
		{parentName}
		ownedCount={parentName ? ownedComponentCount(item, parentName) : 0}
		checked={checked.has(item.key)}
		completedComponents={parentName ? 0 : completedComponentCount(item)}
		automatic={automatic.has(item.key) ||
			(!parentName && item.components.some((part) => automatic.has(part.key)))}
		expanded={!parentName && expanded.includes(item.key)}
		onToggle={() => onToggle(item.key)}
		{onOpenMarket}
		{onBuy}
	/>
{/snippet}

<Table
	{columns}
	{rows}
	rowKey={({ item }) => item.key}
	renderRow={masteryRow}
	virtualize
	emptyMessage="No mastery items match these filters."
	{sortColumn}
	{sortDirection}
	onSort={(key) => onSort(key as SortColumn)}
/>
