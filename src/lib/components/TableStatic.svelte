<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';
	import type { TableColumn } from './table-types';

	let {
		columns,
		children,
		renderHeader,
		sortColumn,
		sortDirection = 'asc',
		onSort = () => {},
		minWidth = '640px',
		bodyClass = '',
		container = $bindable(),
		header = $bindable(),
	}: {
		columns: TableColumn[];
		children: Snippet;
		renderHeader?: Snippet<[column: TableColumn]>;
		sortColumn?: string;
		sortDirection?: 'asc' | 'desc';
		onSort?: (key: string) => void;
		minWidth?: string;
		bodyClass?: string;
		// Callers can measure layout without coupling this shell to virtualization.
		container?: HTMLDivElement;
		header?: HTMLTableSectionElement;
	} = $props();
</script>

<div bind:this={container} class="bg-card/50 border border-border-secondary w-full min-w-0 overflow-x-auto">
	<table class="w-full text-base text-left" style:min-width={minWidth}>
		<thead bind:this={header} class="bg-surface/80 border-b border-border-secondary text-muted-foreground text-sm uppercase tracking-wider">
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
		<tbody class={bodyClass}>
			{@render children()}
		</tbody>
	</table>
</div>
