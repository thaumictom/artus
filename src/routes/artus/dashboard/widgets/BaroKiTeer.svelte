<script lang="ts">
	import { formatTimeLeft } from '$lib/date';
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import type { WorldState } from 'warframe-worldstate-parser';
	import { isBaroActive } from '../baro';
	import { validDate } from '../views/view-types';

	let {
		trader,
		now,
		onOpenInventory,
	}: {
		trader: WorldState['voidTrader'];
		now: number;
		onOpenInventory: () => void;
	} = $props();

	let active = $derived(isBaroActive(trader, now));
	let arrival = $derived(
		validDate(trader.activation) && now < trader.activation.getTime()
			? trader.activation
			: undefined,
	);
</script>

<article class="bg-background p-3 border border-surface min-w-0" aria-labelledby="baro-heading">
	<p id="baro-heading" class="text-muted-foreground text-sm truncate">Baro Ki'Teer</p>
	<div class="flex flex-col gap-2.5">
		<span class="font-medium text-base truncate">{trader.location || 'Unknown relay'}</span>
		<div class="flex flex-1 justify-between items-center text-muted-foreground text-sm">
			{#if active}
				<span>
					Leaves in
					<time datetime={trader.expiry?.toISOString()}>
						{formatTimeLeft(trader.expiry, now)}
					</time>
				</span>
				<Button
					variant="link"
					size="none"
					class="inline-flex items-center gap-1 border-0 text-sm"
					onclick={onOpenInventory}
				>
					View <Icon icon="lucide:arrow-right" class="size-3.5" aria-hidden="true" />
				</Button>
			{:else if arrival}
				<span>
					Arrives in <time datetime={arrival.toISOString()}>{formatTimeLeft(arrival, now)}</time>
				</span>
			{:else}
				<span>Schedule updating</span>
			{/if}
		</div>
	</div>
</article>
