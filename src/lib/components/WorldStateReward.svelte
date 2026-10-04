<script lang="ts">
	import type { Reward } from 'warframe-worldstate-parser';
	import WarframeItem from './WarframeItem.svelte';

	let { reward, completed = false }: { reward?: Reward; completed?: boolean } = $props();
</script>

<!-- countedItems includes uncounted API items at quantity one. -->
{#each reward?.countedItems ?? [] as item}
	<div class="mb-2 last:mb-0">
		<WarframeItem item={item.uniqueName} name={item.type} nameClass={completed ? 'font-medium text-muted-foreground' : 'font-semibold'}>
			{#snippet trailing()}
				{#if item.count > 1}<span class="text-sm text-muted-foreground tabular-nums">×{item.count.toLocaleString()}</span>{/if}
			{/snippet}
		</WarframeItem>
	</div>
{/each}
{#if reward && reward.credits > 0}
	<p class="mt-2 text-sm text-muted-foreground tabular-nums">{reward.credits.toLocaleString()} credits</p>
{:else if !reward || !reward.countedItems.length}
	<p class="text-sm text-muted-foreground">Reward unavailable</p>
{/if}
