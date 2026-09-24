<script lang="ts">
	import Icon from '@iconify/svelte';
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
	} = $props();
</script>

<div class="bg-card/50 border border-border-secondary w-full min-w-0 overflow-x-auto">
	<table class="w-full min-w-[640px] text-sm text-left">
		<thead class="bg-surface/80 text-muted-foreground text-xs uppercase tracking-wider">
			<tr>
				<th scope="col" class="px-4 py-3 w-14"><span class="sr-only">Checked</span></th>
				{#each [{ key: 'name', label: 'Item' }, { key: 'median', label: 'Median' }, { key: 'ducats', label: 'Ducats' }] as column (column.key)}
					<th
						scope="col"
						aria-sort={sortColumn === column.key
							? sortDirection === 'asc'
								? 'ascending'
								: 'descending'
							: 'none'}
						class={column.key === 'name' ? 'px-3 py-3' : 'px-3 py-3 w-28 text-right'}
					>
						<button
							class={`inline-flex w-full items-center gap-1.5 hover:text-foreground focus-visible:outline-2 focus-visible:outline-accent cursor-pointer ${column.key === 'name' ? '' : 'justify-end'}`}
							onclick={() => onSort(column.key as SortColumn)}
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
					</th>
				{/each}
				<th scope="col" class="px-3 py-3 w-36 text-right">Links</th>
			</tr>
		</thead>
		<tbody>
			{#each items as item (item.key)}
				<MasteryRow
					{item}
					checked={checked.has(item.key)}
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
		</tbody>
	</table>
</div>
