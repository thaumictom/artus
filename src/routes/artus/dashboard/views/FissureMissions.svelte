<script lang="ts">
	import { formatTimeLeft } from '$lib/date';
	import { RadioGroup } from 'bits-ui';
	import type { DashboardViewProps } from './view-types';

	let { world, now }: DashboardViewProps = $props();
	let fissures = $derived(world.fissures);
	let fissureFilter = $state<'all' | 'noVoidStorms' | 'normal' | 'steelPath' | 'voidStorm'>(
		'noVoidStorms',
	);
	let eraFilter = $state('all');
	let activeEras = $derived.by(() => {
		const eras = new Map<string, number>();
		for (const fissure of fissures) {
			if (fissure.expiry && fissure.expiry.getTime() > now) {
				eras.set(fissure.tier, Number(fissure.tierNum));
			}
		}
		return [...eras.entries()]
			.sort(([, tierA], [, tierB]) => tierA - tierB)
			.map(([tier]) => tier);
	});

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
</script>

<section class="bg-background border border-surface min-w-0" aria-labelledby="fissures-heading">
	<div class="flex flex-col gap-3 p-4 border-surface border-b">
		<div class="flex flex-wrap items-center gap-3">
			<h2 id="fissures-heading" class="font-medium">Fissure missions</h2>
			<RadioGroup.Root
				aria-label="Filter fissure mission type"
				class="flex p-0.5 border"
				bind:value={fissureFilter}
			>
				<div
					class="inline-flex gap-0.5 *:data-[state=checked]:bg-surface *:px-2.5 *:py-1 *:text-xs *:cursor-pointer"
				>
					<RadioGroup.Item value="all">All</RadioGroup.Item>
					<RadioGroup.Item value="noVoidStorms">No Void Storms</RadioGroup.Item>
					<RadioGroup.Item value="normal">Normal</RadioGroup.Item>
					<RadioGroup.Item value="steelPath">Steel Path</RadioGroup.Item>
					<RadioGroup.Item value="voidStorm">Void Storm</RadioGroup.Item>
				</div>
			</RadioGroup.Root>
			<span class="ml-auto text-muted-foreground text-xs">{activeFissures.length} active</span>
		</div>
		<div class="flex flex-wrap items-center gap-2">
			<span class="text-muted-foreground text-xs">Era</span>
			<RadioGroup.Root
				aria-label="Filter fissure era"
				class="flex p-0.5 border"
				bind:value={eraFilter}
			>
				<div
					class="inline-flex flex-wrap gap-0.5 *:data-[state=checked]:bg-surface *:px-2.5 *:py-1 *:text-xs *:cursor-pointer"
				>
					<RadioGroup.Item value="all">All</RadioGroup.Item>
					{#each activeEras as era (era)}
						<RadioGroup.Item value={era}>{era}</RadioGroup.Item>
					{/each}
				</div>
			</RadioGroup.Root>
		</div>
	</div>

	<ul class="divide-y divide-surface">
		{#each activeFissures as fissure (fissure.id)}
			<li class="gap-x-3 grid grid-cols-[minmax(0,1fr)_auto] px-4 py-2.5 text-sm">
				<div class="flex flex-col gap-1">
					<div class="flex items-center gap-1">
						<span class="bg-surface px-2 py-0.5 text-muted-foreground text-xs">
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
</section>
