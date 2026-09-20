<script lang="ts">
	import type { WorldState } from 'warframe-worldstate-parser';

	let { world, now }: { world: WorldState; now: number } = $props();

	let cycles = $derived([
		{ label: 'Earth', cycle: world.earthCycle },
		{ label: 'Cetus', cycle: world.cetusCycle },
		{ label: 'Cambion Drift', cycle: world.cambionCycle },
		{ label: 'Orb Vallis', cycle: world.vallisCycle },
		{ label: 'Zariman', cycle: world.zarimanCycle },
		{ label: 'Duviri', cycle: world.duviriCycle },
	]);

	function formatState(state: string) {
		return state.charAt(0).toUpperCase() + state.slice(1);
	}

	function timeLeft(expiry: Date | undefined) {
		if (!expiry) return 'Unavailable';

		const totalSeconds = Math.max(0, Math.ceil((expiry.getTime() - now) / 1000));
		const hours = Math.floor(totalSeconds / 3600);
		const minutes = Math.floor((totalSeconds % 3600) / 60);
		const seconds = totalSeconds % 60;

		return hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m ${seconds}s`;
	}
</script>

<section aria-label="World cycles" class="gap-2 grid grid-cols-2 md:grid-cols-3 xl:grid-cols-6">
	{#each cycles as { label, cycle } (label)}
		<div class="bg-background p-3 border border-surface min-w-0">
			<p class="text-muted-foreground text-xs truncate">{label}</p>
			<div class="flex justify-between items-baseline gap-2 mt-1">
				<span class="font-medium text-sm truncate">{formatState(cycle.state)}</span>
				<time
					class="text-muted-foreground text-xs tabular-nums whitespace-nowrap"
					datetime={cycle.expiry?.toISOString()}
				>
					{timeLeft(cycle.expiry)}
				</time>
			</div>
		</div>
	{/each}
</section>
