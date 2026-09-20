<script lang="ts">
	import type { WorldState } from 'warframe-worldstate-parser';
	import { formatTimeLeft } from '$lib/date';
	import { getDashboardCycles } from '../cycles';
	import CycleWave from './CycleWave.svelte';

	let { world, now }: { world: WorldState; now: number } = $props();

	let cycles = $derived(getDashboardCycles(world, now));

	function formatState(state: string) {
		return state.charAt(0).toUpperCase() + state.slice(1);
	}

</script>

<section aria-label="World cycles" class="gap-2 grid grid-cols-2 md:grid-cols-3 xl:grid-cols-6">
	{#each cycles as { key, label, cycle } (key)}
		<div class="bg-background p-3 border border-surface min-w-0">
			<p class="text-muted-foreground text-xs truncate">{label}</p>
			<CycleWave {label} state={cycle.state} nextState={cycle.nextState} progress={cycle.progress} />
			<div class="flex justify-between items-baseline gap-2">
				<span class="font-medium text-sm truncate">{formatState(cycle.state)}</span>
				<time
					class="text-muted-foreground text-xs tabular-nums whitespace-nowrap"
					datetime={cycle.expiry?.toISOString()}
				>
					{formatTimeLeft(cycle.expiry, now)}
				</time>
			</div>
		</div>
	{/each}
</section>
