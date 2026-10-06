<script lang="ts" generics="Row">
	import Icon from '@iconify/svelte';
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

	let container: HTMLDivElement;
	let header: HTMLTableSectionElement;
	let scrollElement = $state<HTMLElement | null>(null);
	let fallbackAll = $state(false);
	const rowVirtualizer = createVirtualizer<HTMLElement, HTMLTableRowElement>({
		count: 0,
		getScrollElement: () => scrollElement,
		estimateSize: () => 76,
		getItemKey: (index) => rowKey(rows[index]),
		overscan: 6,
		enabled: false,
	});
	const virtualRows = $derived($rowVirtualizer.getVirtualItems());
	const topPadding = $derived(virtualRows.length ? virtualRows[0].start - $rowVirtualizer.options.scrollMargin : 0);
	const bottomPadding = $derived(virtualRows.length
		? $rowVirtualizer.getTotalSize() - (virtualRows[virtualRows.length - 1].end - $rowVirtualizer.options.scrollMargin)
		: 0);
	const measureRow = (element: HTMLTableRowElement) => $rowVirtualizer.measureElement(element);

	$effect(() => {
		if (!virtualize) return;
		const currentRows = rows;
		// Updating the store must not make this effect depend on its own publication.
		untrack(() => $rowVirtualizer.setOptions({
			count: currentRows.length,
			getItemKey: (index) => rowKey(currentRows[index]),
		}));
	});

	onMount(() => {
		if (!virtualize) return;
		// OverlayScrollbars uses this existing element as its viewport after initialization.
		scrollElement = container.closest<HTMLElement>('[data-overlayscrollbars-contents]');
		if (!scrollElement) {
			fallbackAll = true;
			return;
		}
		const updateMargin = () => {
			const margin = header.getBoundingClientRect().bottom - scrollElement!.getBoundingClientRect().top + scrollElement!.scrollTop;
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

<div bind:this={container} class="bg-card/50 border border-border-secondary w-full min-w-0 overflow-x-auto">
	<table class="w-full text-base text-left" style:min-width={minWidth}>
		<thead bind:this={header} class="bg-surface/80 text-muted-foreground text-sm uppercase tracking-wider">
			<tr>
				{#each columns as column (column.key)}
					<th
						scope="col"
						class={`px-3 py-3 ${column.align === 'right' ? 'text-right' : ''} ${column.class ?? ''}`}
						aria-sort={column.sortable
							? sortColumn === column.key
								? sortDirection === 'asc'
									? 'ascending'
									: 'descending'
								: 'none'
							: undefined}
					>
						{#if renderHeader}
							{@render renderHeader(column)}
						{:else if column.sortable}
							<button
							class={`inline-flex w-full items-center gap-1.5 hover:text-foreground focus-visible:outline-2 focus-visible:outline-accent cursor-pointer ${column.align === 'right' ? 'justify-end' : ''}`}
							onclick={() => onSort(column.key)}
						>
								{column.label}
								<Icon
									icon={sortColumn === column.key
										? sortDirection === 'asc'
											? 'material-symbols:arrow-upward-rounded'
											: 'material-symbols:arrow-downward-rounded'
										: 'material-symbols:unfold-more-rounded'}
									class={`size-4 ${sortColumn === column.key ? 'text-accent' : 'opacity-50'}`}
								/>
							</button>
						{:else}{column.label}{/if}
					</th>
				{/each}
			</tr>
		</thead>
		<tbody>
			{#if virtualize && rows.length && scrollElement && virtualRows.length}
				{#if topPadding > 0}<tr aria-hidden="true"><td colspan={columns.length} style:height={`${topPadding}px`} class="!p-0 !border-0"></td></tr>{/if}
				{#each virtualRows as virtualRow (virtualRow.key)}
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
		</tbody>
	</table>
</div>
