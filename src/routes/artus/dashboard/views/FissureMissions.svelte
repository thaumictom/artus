<script lang="ts">
	import { formatTimeLeft } from '$lib/date';
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import type { DashboardViewProps } from './view-types';

	const fissureFilterOptions = [
		{ value: 'all', label: 'All' },
		{ value: 'noVoidStorms', label: 'No Void Storms' },
		{ value: 'normal', label: 'Normal' },
		{ value: 'steelPath', label: 'Steel Path' },
		{ value: 'voidStorm', label: 'Void Storm' },
	] as const;
	type FissureFilter = (typeof fissureFilterOptions)[number]['value'];

	let { world, now }: DashboardViewProps = $props();
	let fissures = $derived(world.fissures);
	let fissureFilter = $state<FissureFilter>('noVoidStorms');
	let eraFilter = $state('all');
	let activeEras = $derived.by(() => {
		const eras = new Map<string, number>();
		for (const fissure of fissures) {
			if (fissure.expiry && fissure.expiry.getTime() > now) {
				eras.set(fissure.tier, Number(fissure.tierNum));
			}
		}
		return [...eras.entries()].sort(([, tierA], [, tierB]) => tierA - tierB).map(([tier]) => tier);
	});
	let eraFilterOptions = $derived([
		{ value: 'all', label: 'All' },
		...activeEras.map((era) => ({ value: era, label: era })),
	]);

	let activeFissures = $derived(
		fissures
			.filter((fissure) => fissure.expiry && fissure.expiry.getTime() > now)
			.filter((fissure) => {
				if (fissureFilter === 'noVoidStorms') return !fissure.isStorm;
				if (fissureFilter === 'normal') return !fissure.isHard && !fissure.isStorm;
				if (fissureFilter === 'steelPath') return fissure.isHard;
				if (fissureFilter === 'voidStorm') return fissure.isStorm;
				return true;
			})
			.filter((fissure) => eraFilter === 'all' || fissure.tier === eraFilter)
			.sort(
				(a, b) =>
					Number(a.tierNum) - Number(b.tierNum) ||
					(a.expiry?.getTime() ?? 0) - (b.expiry?.getTime() ?? 0),
			),
	);

	function eraLabelClass(tier: string) {
		const base = 'border px-2 py-0.5 font-medium text-xs';
		switch (tier.toLowerCase()) {
			case 'lith':
				return `${base} bg-[#d08770]/20 border-[#d08770]/50 text-[#d08770]`;
			case 'meso':
				return `${base} bg-[#4c588a] border-[#7b88a1] text-[#eceff4]`;
			case 'neo':
				return `${base} bg-[#d8dee9]/15 border-[#d8dee9]/50 text-[#d8dee9]`;
			case 'axi':
				return `${base} bg-[#ebcb8b]/20 border-[#ebcb8b]/50 text-[#ebcb8b]`;
			case 'requiem':
				return `${base} bg-[#bf616a]/20 border-[#bf616a]/50 text-[#bf616a]`;
			case 'omnia':
				return `${base} omnia-era border-white/40 text-white`;
			default:
				return `${base} bg-surface text-muted-foreground`;
		}
	}
</script>

<div class="flex flex-col gap-2">
	<div class="flex flex-wrap items-center gap-2">
		<RadioGroup
			label="Filter fissure mission type"
			options={fissureFilterOptions}
			bind:value={fissureFilter}
		/>
		<div class="flex-1 bg-surface h-px"></div>
		<RadioGroup
			label="Filter fissure era"
			options={eraFilterOptions}
			bind:value={eraFilter}
		/>
	</div>
</div>

<ul>
	{#each activeFissures as fissure (fissure.id)}
		<li class="gap-x-3 grid grid-cols-[minmax(0,1fr)_auto] px-1.5 py-3 not-last:border-b text-sm">
			<div class="flex flex-col gap-1">
				<div class="flex items-center gap-1.5">
					<span class={eraLabelClass(fissure.tier)}>
						{fissure.tier}
					</span>
					<p class="truncate">
						{fissure.missionType}
					</p>
				</div>
				<p class="text-muted-foreground text-xs truncate">
					{fissure.node}
					{#if fissure.isStorm}
						· Void Storm{/if}
					{#if fissure.isHard}
						· Steel Path{/if}
				</p>
			</div>
			<div class="flex items-center gap-2">
				{#if fissure.expiry}
					<time
						class="tabular-nums text-muted-foreground text-sm whitespace-nowrap"
						datetime={fissure.expiry.toISOString()}
					>
						{formatTimeLeft(fissure.expiry, now)}
					</time>
				{/if}
			</div>
		</li>
	{:else}
		<li class="p-4 text-muted-foreground text-sm">No active fissures in this snapshot.</li>
	{/each}
</ul>

<style>
	.omnia-era {
		background: linear-gradient(100deg, #d0877060, #4c566a60, #d8dee960, #ebcb8b60, #bf616a60);
	}
</style>
