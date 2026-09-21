<script lang="ts">
	import { formatTimeLeft } from '$lib/date';
	import type { WorldState } from 'warframe-worldstate-parser';

	let { trader, now }: { trader: WorldState['voidTrader']; now: number } = $props();

	let arrival = $derived(
		trader.activation && now < trader.activation.getTime() ? trader.activation : undefined,
	);
</script>

<article class="bg-background p-3 border border-surface min-w-0" aria-labelledby="baro-heading">
	<p id="baro-heading" class="text-muted-foreground text-xs truncate">Baro Ki'Teer</p>
	<div class="flex flex-wrap justify-between items-baseline gap-x-2 gap-y-2.5">
		<span class="font-medium text-sm truncate">{trader.location || 'Unknown relay'}</span>
		<span class="tabular-nums text-muted-foreground text-xs whitespace-nowrap">
			{#if arrival}
				Arrives in <time datetime={arrival.toISOString()}>{formatTimeLeft(arrival, now)}</time>
			{:else}
				Schedule updating
			{/if}
		</span>
	</div>
</article>
