<script lang="ts">
	import { formatTimeLeft } from '$lib/date';
	import type { WorldState } from 'warframe-worldstate-parser';

	let { fissures, now }: { fissures: WorldState['fissures']; now: number } = $props();

	let activeFissures = $derived(
		fissures
			.filter((fissure) => fissure.expiry && fissure.expiry.getTime() > now)
			.sort(
				(a, b) =>
					Number(a.tierNum) - Number(b.tierNum) ||
					(a.expiry?.getTime() ?? 0) - (b.expiry?.getTime() ?? 0),
			),
	);
</script>

<section class="bg-background border border-surface min-w-0" aria-labelledby="fissures-heading">
	<div class="flex justify-between items-baseline gap-3 p-4 border-b border-surface">
		<h2 id="fissures-heading" class="font-medium">Fissure missions</h2>
		<span class="text-muted-foreground text-xs">{activeFissures.length} active</span>
	</div>

	<ul class="divide-y divide-surface max-h-80 overflow-y-auto">
		{#each activeFissures as fissure (fissure.id)}
			<li class="gap-x-3 grid grid-cols-[auto_minmax(0,1fr)_auto] px-4 py-2.5 text-sm">
				<span class="font-medium">{fissure.tier}</span>
				<div class="min-w-0">
					<p class="truncate">{fissure.node}</p>
					<p class="text-muted-foreground text-xs truncate">
						{fissure.missionType}
						{#if fissure.isStorm} · Void Storm{/if}
						{#if fissure.isHard} · Steel Path{/if}
					</p>
				</div>
				{#if fissure.expiry}
					<time
						class="text-muted-foreground text-xs tabular-nums whitespace-nowrap"
						datetime={fissure.expiry.toISOString()}
					>
						{formatTimeLeft(fissure.expiry, now)}
					</time>
				{/if}
			</li>
		{:else}
			<li class="p-4 text-muted-foreground text-sm">No active fissures in this snapshot.</li>
		{/each}
	</ul>
</section>
