<script lang="ts">
	import type { WorldState } from 'warframe-worldstate-parser';
	import { formatTimeLeft } from '$lib/date';
	import { getDashboardCycles } from '../cycles';
	import BaroKiTeer from './BaroKiTeer.svelte';

	let { world, now }: { world: WorldState; now: number } = $props();

	let cycles = $derived(getDashboardCycles(world, now));

	function formatState(state: string) {
		return state.charAt(0).toUpperCase() + state.slice(1);
	}
</script>

<section aria-label="World cycles" class="gap-2 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3">
	{#each cycles as { key, label, cycle } (key)}
		<div class="bg-background p-3 border border-surface min-w-0">
			<p class="text-muted-foreground text-xs truncate">{label}</p>
			<div class="flex flex-wrap justify-between items-baseline gap-x-2 gap-y-1">
				<span class="font-medium text-sm truncate">{formatState(cycle.state)}</span>
				<span class="ml-auto tabular-nums text-muted-foreground text-xs whitespace-nowrap">
					{#if cycle.expiry}
						{formatState(cycle.nextState)} in
						<time datetime={cycle.expiry.toISOString()}>{formatTimeLeft(cycle.expiry, now)}</time>
					{:else}
						Unavailable
					{/if}
				</span>
			</div>
			<div
				class="items-center gap-0.5 grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] my-2 mt-3"
				aria-hidden="true"
			>
				<!-- Each half ends at now; the dot occupies its own column and never covers a phase. -->
				{#each [0, 50] as offset}
					{#if offset === 50}
						<div class="bg-foreground w-1 h-3"></div>
					{/if}
					<div class="relative h-1.5 overflow-hidden">
						{#each cycle.segments as segment}
							<div
								class="absolute inset-y-0"
								style:left={`calc(${(segment.left - offset) * 2}% + 1px)`}
								style:width={`max(0px, calc(${segment.width * 2}% - 2px))`}
								style:background-color={segment.color}
							></div>
						{/each}
					</div>
				{/each}
			</div>
		</div>
	{/each}
	<BaroKiTeer trader={world.voidTrader} {now} />
</section>
