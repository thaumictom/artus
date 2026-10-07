<script lang="ts">
	import Currency from '$lib/components/Currency.svelte';
	import type { Reward } from 'warframe-worldstate-parser';
	import WarframeItem from './WarframeItem.svelte';

	let { reward, completed = false }: { reward?: Reward; completed?: boolean } = $props();
</script>

<!-- countedItems includes uncounted API items at quantity one. -->
{#each reward?.countedItems ?? [] as item}
	<div>
		<WarframeItem
			item={item.uniqueName}
			name={item.type}
			nameClass={completed ? 'font-medium text-muted-foreground' : 'font-semibold'}
		>
			{#snippet trailing()}
				{#if item.count > 1}<span class="tabular-nums text-muted-foreground text-sm">
						×{item.count.toLocaleString()}
					</span>{/if}
			{/snippet}
		</WarframeItem>
	</div>
{/each}
{#if reward && reward.credits > 0}
	<p class="tabular-nums text-muted-foreground text-sm">
		<Currency value={reward.credits} currency="credits" />
	</p>
{:else if !reward || !reward.countedItems.length}
	<p class="text-muted-foreground text-sm">Reward unavailable</p>
{/if}
