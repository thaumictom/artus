<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';

	export type TableColumn = {
		key: string;
		label: string;
		sortable?: boolean;
		align?: 'left' | 'right';
		class?: string;
	};

	let {
		columns,
		children,
		sortColumn,
		sortDirection = 'asc',
		onSort = () => {},
		minWidth = '640px',
	}: {
		columns: TableColumn[];
		children: Snippet;
		sortColumn?: string;
		sortDirection?: 'asc' | 'desc';
		onSort?: (key: string) => void;
		minWidth?: string;
	} = $props();
</script>

<div class="bg-card/50 border border-border-secondary w-full min-w-0 overflow-x-auto">
	<table class="w-full text-sm text-left" style:min-width={minWidth}>
		<thead class="bg-surface/80 text-muted-foreground text-xs uppercase tracking-wider">
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
						{#if column.sortable}
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
		<tbody>{@render children()}</tbody>
	</table>
</div>
