<script lang="ts">
	import Table, { type TableColumn } from '$lib/components/Table.svelte';
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
		ownedComponentCount: (component: MasteryItem, parentName: string) => number;
		completedComponentCount: (item: MasteryItem) => number;
	} = $props();

	const columns: TableColumn[] = [
		{ key: 'checked', label: '', class: 'w-18 px-4' },
		{ key: 'name', label: 'Item', sortable: true },
		{ key: 'median', label: 'Median', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'ducats', label: 'Ducats', sortable: true, align: 'right', class: 'w-28' },
		{ key: 'links', label: 'Links', align: 'right', class: 'w-36' },
	];
</script>

<Table {columns} {sortColumn} {sortDirection} onSort={(key) => onSort(key as SortColumn)}>
			{#each items as item (item.key)}
				<MasteryRow
					{item}
					checked={checked.has(item.key)}
					completedComponents={completedComponentCount(item)}
					automatic={automatic.has(item.key) ||
						item.components.some((part) => automatic.has(part.key))}
					expanded={expanded.includes(item.key)}
					onToggle={() => onToggle(item.key)}
					{onOpenMarket}
				/>
				{#if expanded.includes(item.key)}
					{#each item.components as component (component.key)}
						<MasteryRow
							item={component}
							parentName={item.name}
							ownedCount={ownedComponentCount(component, item.name)}
							checked={checked.has(component.key)}
							automatic={automatic.has(component.key)}
							{onOpenMarket}
						/>
					{/each}
				{/if}
			{:else}
				<tr>
					<td colspan="5" class="px-4 py-10 text-muted-foreground text-center">
						No mastery items match these filters.
					</td>
				</tr>
			{/each}
</Table>
