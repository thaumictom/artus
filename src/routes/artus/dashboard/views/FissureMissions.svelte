<script lang="ts">
	import { formatTimeLeft } from '$lib/date';
	import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
	import { mode } from 'mode-watcher';
	import { RadioGroup } from 'bits-ui';
	import type { DashboardViewProps } from './view-types';

	let { world, now }: DashboardViewProps = $props();
	let fissures = $derived(world.fissures);
	let scrollbarTheme = $derived.by(() =>
		mode.current === 'light' ? 'os-theme-dark' : 'os-theme-light',
	);
	let fissureFilter = $state<'all' | 'normal' | 'steelPath'>('all');

	let activeFissures = $derived(
		fissures
			.filter((fissure) => fissure.expiry && fissure.expiry.getTime() > now)
			.filter((fissure) => {
				if (fissureFilter === 'normal') return !fissure.isHard;
				if (fissureFilter === 'steelPath') return fissure.isHard;
				return true;
			})
			.sort(
				(a, b) =>
					Number(a.tierNum) - Number(b.tierNum) ||
					(a.expiry?.getTime() ?? 0) - (b.expiry?.getTime() ?? 0),
			),
	);
</script>

<section class="bg-background border border-surface min-w-0" aria-labelledby="fissures-heading">
	<div class="flex flex-wrap items-center gap-3 p-4 border-surface border-b">
		<h2 id="fissures-heading" class="font-medium">Fissure missions</h2>
		<RadioGroup.Root
			aria-label="Filter fissure missions"
			class="flex p-0.5 border"
			bind:value={fissureFilter}
		>
			<div
				class="inline-flex gap-0.5 *:data-[state=checked]:bg-surface *:px-2.5 *:py-1 *:text-xs *:cursor-pointer"
			>
				<RadioGroup.Item value="all">All</RadioGroup.Item>
				<RadioGroup.Item value="normal">Normal</RadioGroup.Item>
				<RadioGroup.Item value="steelPath">Steel Path</RadioGroup.Item>
			</div>
		</RadioGroup.Root>
		<span class="ml-auto text-muted-foreground text-xs">{activeFissures.length} active</span>
	</div>

	<OverlayScrollbarsComponent
		defer
		options={{ scrollbars: { theme: scrollbarTheme, autoHide: 'move' } }}
		class="max-h-80"
	>
		<ul class="divide-y divide-surface">
			{#each activeFissures as fissure (fissure.id)}
				<li class="gap-x-3 grid grid-cols-[auto_minmax(0,1fr)_auto] px-4 py-2.5 text-sm">
					<span class="font-medium">{fissure.tier}</span>
					<div class="min-w-0">
						<p class="truncate">{fissure.node}</p>
						<p class="text-muted-foreground text-xs truncate">
							{fissure.missionType}
							{#if fissure.isStorm}
								· Void Storm{/if}
							{#if fissure.isHard}
								· Steel Path{/if}
						</p>
					</div>
					{#if fissure.expiry}
						<time
							class="tabular-nums text-muted-foreground text-xs whitespace-nowrap"
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
	</OverlayScrollbarsComponent>
</section>
