<script lang="ts" generics="TData, TValue">
	import {
		type ColumnDef,
		type ColumnFiltersState,
		type FilterFn,
		type RowSelectionState,
		type SortingState,
		getCoreRowModel,
		getFilteredRowModel,
		getSortedRowModel,
	} from '@tanstack/table-core';
	import { createSvelteTable, FlexRender } from '$lib/components/ui/data-table/index.js';
	import * as Table from '$lib/components/ui/table/index.js';

	type DataTableProps<TData, TValue> = {
		columns: ColumnDef<TData, TValue>[];
		data: TData[];
	};

	let { data, columns }: DataTableProps<TData, TValue> = $props();
	let sorting = $state<SortingState>([]);
	let columnFilters = $state<ColumnFiltersState>([]);
	let globalFilter = $state('');
	let rowSelection = $state<RowSelectionState>({});

	const globalTextFilter: FilterFn<TData> = (row, _columnId, filterValue) => {
		const searchValue = String(filterValue ?? '').trim().toLowerCase();
		if (!searchValue) return true;

		return row
			.getVisibleCells()
			.some((cell) => String(cell.getValue() ?? '').toLowerCase().includes(searchValue));
	};

	const table = createSvelteTable({
		get data() {
			return data;
		},
		get columns() {
			return columns;
		},
		state: {
			get sorting() {
				return sorting;
			},
			get columnFilters() {
				return columnFilters;
			},
			get globalFilter() {
				return globalFilter;
			},
			get rowSelection() {
				return rowSelection;
			},
		},
		getRowId: (row, index) => {
			const rowRecord = row as Record<string, unknown>;
			const stableId = rowRecord.uniqueName ?? rowRecord.id;

			if (typeof stableId === 'string' && stableId.length > 0) return stableId;
			return `${index}`;
		},
		enableRowSelection: true,
		onSortingChange: (updater) => {
			sorting = updater instanceof Function ? updater(sorting) : updater;
		},
		onColumnFiltersChange: (updater) => {
			columnFilters = updater instanceof Function ? updater(columnFilters) : updater;
		},
		onGlobalFilterChange: (updater) => {
			globalFilter = updater instanceof Function ? updater(globalFilter) : String(updater ?? '');
		},
		onRowSelectionChange: (updater) => {
			rowSelection = updater instanceof Function ? updater(rowSelection) : updater;
		},
		globalFilterFn: globalTextFilter,
		getCoreRowModel: getCoreRowModel(),
		getFilteredRowModel: getFilteredRowModel(),
		getSortedRowModel: getSortedRowModel(),
	});

	const categoryColumn = $derived(table.getColumn('category'));
	const markedCount = $derived(table.getSelectedRowModel().rows.length);
	const categoryOptions = $derived.by(() => {
		const uniqueCategories = new Set<string>();

		for (const row of data as Array<Record<string, unknown>>) {
			const category = row.category;
			if (typeof category === 'string' && category.length > 0) uniqueCategories.add(category);
		}

		return [...uniqueCategories].sort((a, b) => a.localeCompare(b));
	});

	function sortIndicator(sortDirection: false | 'asc' | 'desc') {
		if (sortDirection === 'asc') return ' (asc)';
		if (sortDirection === 'desc') return ' (desc)';
		return '';
	}

	function onCategoryChange(event: Event) {
		const value = (event.currentTarget as HTMLSelectElement).value;
		categoryColumn?.setFilterValue(value || undefined);
	}

	function clearFilters() {
		globalFilter = '';
		columnFilters = [];
	}

	function clearMarked() {
		rowSelection = {};
	}
</script>

<div class="flex flex-wrap items-center gap-2 py-4">
	<input
		class="px-2 py-1 border rounded min-w-56"
		type="search"
		placeholder="Search all columns..."
		bind:value={globalFilter}
	/>

	{#if categoryColumn}
		<select
			class="px-2 py-1 border rounded"
			value={String(categoryColumn.getFilterValue() ?? '')}
			onchange={onCategoryChange}
		>
			<option value="">All categories</option>
			{#each categoryOptions as category}
				<option value={category}>{category}</option>
			{/each}
		</select>
	{/if}

	<button type="button" class="px-2 py-1 border rounded" onclick={clearFilters}>Reset filters</button>
	<div class="ml-auto text-sm">{markedCount} marked</div>
	<button
		type="button"
		class="px-2 py-1 border rounded disabled:opacity-50"
		disabled={markedCount === 0}
		onclick={clearMarked}
	>
		Clear marked
	</button>
</div>

<div class="border rounded-md">
	<Table.Root>
		<Table.Header>
			{#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
				<Table.Row>
					<Table.Head class="w-20">Marked</Table.Head>
					{#each headerGroup.headers as header (header.id)}
						<Table.Head colspan={header.colSpan}>
							{#if !header.isPlaceholder}
								{#if header.column.getCanSort()}
									<button
										type="button"
										class="w-full text-left"
										onclick={() => header.column.toggleSorting(header.column.getIsSorted() === 'asc')}
									>
										<FlexRender
											content={header.column.columnDef.header}
											context={header.getContext()}
										/>{sortIndicator(header.column.getIsSorted())}
									</button>
								{:else}
									<FlexRender
										content={header.column.columnDef.header}
										context={header.getContext()}
									/>
								{/if}
							{/if}
						</Table.Head>
					{/each}
				</Table.Row>
			{/each}
		</Table.Header>
		<Table.Body>
			{#each table.getRowModel().rows as row (row.id)}
				<Table.Row data-state={row.getIsSelected() && 'selected'}>
					<Table.Cell>
						<input
							type="checkbox"
							checked={row.getIsSelected()}
							onchange={(event) => row.toggleSelected((event.currentTarget as HTMLInputElement).checked)}
							aria-label={`Mark row ${row.id}`}
						/>
					</Table.Cell>
					{#each row.getVisibleCells() as cell (cell.id)}
						<Table.Cell>
							<FlexRender content={cell.column.columnDef.cell} context={cell.getContext()} />
						</Table.Cell>
					{/each}
				</Table.Row>
			{:else}
				<Table.Row>
					<Table.Cell colspan={columns.length + 1} class="h-24 text-center">No results.</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>
</div>
