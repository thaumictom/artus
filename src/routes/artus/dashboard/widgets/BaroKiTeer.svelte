<script lang="ts">
	import { formatTimeLeft } from '$lib/date';
	import type { WorldState } from 'warframe-worldstate-parser';

	let { trader, now }: { trader: WorldState['voidTrader']; now: number } = $props();

	let isActive = $derived(
		Boolean(
			trader.activation &&
				trader.expiry &&
				now >= trader.activation.getTime() &&
				now < trader.expiry.getTime(),
		),
	);
	let isUpcoming = $derived(Boolean(trader.activation && now < trader.activation.getTime()));
	let target = $derived(isActive ? trader.expiry : isUpcoming ? trader.activation : undefined);
</script>

<section class="bg-background p-4 border border-surface" aria-labelledby="baro-heading">
	<div class="flex justify-between items-start gap-3">
		<div>
			<h2 id="baro-heading" class="font-medium">Baro Ki'Teer</h2>
			<p class="mt-1 text-muted-foreground text-sm">{trader.location}</p>
		</div>
		<span class={isActive ? 'text-accent text-xs' : 'text-muted-foreground text-xs'}>
			{isActive ? 'Available' : 'Away'}
		</span>
	</div>

	<div class="mt-5">
		<p class="text-muted-foreground text-xs">
			{isActive ? 'Leaves in' : isUpcoming ? 'Arrives in' : 'Schedule updating'}
		</p>
		{#if target}
			<time
				class="block mt-1 font-medium text-lg tabular-nums"
				datetime={target.toISOString()}
			>
				{formatTimeLeft(target, now)}
			</time>
		{/if}
	</div>
</section>
