<script lang="ts" generics="Row">
	import TableStatic from './TableStatic.svelte';
	import type { Snippet } from 'svelte';
	import { onMount, untrack } from 'svelte';
	import { createVirtualizer } from '@tanstack/svelte-virtual';
	import type { TableColumn } from './table-types';

	let {
		columns,
		rows,
		renderRow,
		renderHeader,
		rowKey,
		emptyMessage = 'No items to show.',
		sortColumn,
		sortDirection = 'asc',
		onSort = () => {},
		minWidth = '640px',
		virtualize = false,
	}: {
		columns: TableColumn[];
		rows: Row[];
		renderRow: Snippet<[row: Row, index: number, measureRow: (element: HTMLTableRowElement) => void]>;
		renderHeader?: Snippet<[column: TableColumn]>;
		rowKey: (row: Row) => string;
		emptyMessage?: string;
		sortColumn?: string;
		sortDirection?: 'asc' | 'desc';
		onSort?: (key: string) => void;
		minWidth?: string;
		virtualize?: boolean;
	} = $props();

	let container = $state<HTMLDivElement>();
	let header = $state<HTMLTableSectionElement>();
	let scrollElement = $state<HTMLElement | null>(null);
	let fallbackAll = $state(false);
	const keyAt = (currentRows: Row[], index: number) =>
		index >= 0 && index < currentRows.length ? rowKey(currentRows[index]) : `missing-row-${index}`;
	const rowVirtualizer = createVirtualizer<HTMLElement, HTMLTableRowElement>({
		count: 0,
		getScrollElement: () => scrollElement,
		estimateSize: () => 76,
		getItemKey: (index) => keyAt(rows, index),
		overscan: 6,
		enabled: false,
	});
	const virtualRows = $derived($rowVirtualizer.getVirtualItems());
	// Props can change before the virtualizer publishes its new indices and keys.
	const visibleVirtualRows = $derived(virtualRows.filter(({ index, key }) =>
		index < rows.length && key === keyAt(rows, index)));
	const topPadding = $derived(visibleVirtualRows.length ? visibleVirtualRows[0].start - $rowVirtualizer.options.scrollMargin : 0);
	const bottomPadding = $derived(visibleVirtualRows.length
		? $rowVirtualizer.getTotalSize() - (visibleVirtualRows[visibleVirtualRows.length - 1].end - $rowVirtualizer.options.scrollMargin)
		: 0);
	const measureRow = (element: HTMLTableRowElement) => $rowVirtualizer.measureElement(element);

	$effect(() => {
		if (!virtualize) return;
		const currentRows = rows;
		// Updating the store must not make this effect depend on its own publication.
		untrack(() => $rowVirtualizer.setOptions({
			count: currentRows.length,
			getItemKey: (index) => keyAt(currentRows, index),
		}));
	});

	onMount(() => {
		if (!virtualize || !container || !header) return;
		const tableHeader = header;
		// OverlayScrollbars uses this existing element as its viewport after initialization.
		scrollElement = container.closest<HTMLElement>('[data-overlayscrollbars-contents]');
		if (!scrollElement) {
			fallbackAll = true;
			return;
		}
		const updateMargin = () => {
			const margin = tableHeader.getBoundingClientRect().bottom - scrollElement!.getBoundingClientRect().top + scrollElement!.scrollTop;
			if (margin !== $rowVirtualizer.options.scrollMargin)
				$rowVirtualizer.setOptions({ scrollMargin: margin });
		};
		updateMargin();
		$rowVirtualizer.setOptions({ getScrollElement: () => scrollElement, enabled: true });
		// Keep the first paint small while OverlayScrollbars finishes initializing.
		const fallbackTimer = window.setTimeout(() => {
			if ($rowVirtualizer.getVirtualItems().length === 0) fallbackAll = true;
		}, 500);
		const observer = new ResizeObserver(updateMargin);
		observer.observe(header);
		observer.observe(container.parentElement!);
		scrollElement.addEventListener('scroll', updateMargin, { passive: true });
		return () => {
			clearTimeout(fallbackTimer);
			observer.disconnect();
			scrollElement?.removeEventListener('scroll', updateMargin);
		};
	});
</script>

<TableStatic
	{columns}
	{renderHeader}
	{sortColumn}
	{sortDirection}
	{onSort}
	{minWidth}
	bind:container
	bind:header
>
	{#if virtualize && rows.length && scrollElement && visibleVirtualRows.length}
		{#if topPadding > 0}<tr aria-hidden="true"><td colspan={columns.length} style:height={`${topPadding}px`} class="!p-0 !border-0"></td></tr>{/if}
		{#each visibleVirtualRows as virtualRow (virtualRow.key)}
			{@render renderRow(rows[virtualRow.index], virtualRow.index, measureRow)}
		{/each}
		{#if bottomPadding > 0}<tr aria-hidden="true"><td colspan={columns.length} style:height={`${bottomPadding}px`} class="!p-0 !border-0"></td></tr>{/if}
	{:else if rows.length}
		{#each virtualize && !fallbackAll ? rows.slice(0, 20) : rows as row, index (rowKey(row))}
			{@render renderRow(row, index, measureRow)}
		{/each}
		{#if virtualize && !fallbackAll && rows.length > 20}
			<tr aria-hidden="true"><td colspan={columns.length} style:height={`${(rows.length - 20) * 76}px`} class="!p-0 !border-0"></td></tr>
		{/if}
	{:else}
		<tr>
			<td colspan={columns.length} class="px-4 py-10 text-muted-foreground text-center">
				{emptyMessage}
			</td>
		</tr>
	{/if}
</TableStatic>
