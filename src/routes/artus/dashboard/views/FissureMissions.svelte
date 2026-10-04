<script lang="ts">
	import { formatTimeLeft } from '$lib/date';
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import ToggleGroup from '$lib/components/ToggleGroup.svelte';
	import Table from '$lib/components/Table.svelte';
	import type { TableColumn } from '$lib/components/table-types';
	import type { DashboardViewProps } from './view-types';

	const columns: TableColumn[] = [
		{ key: 'era', label: 'Era', class: 'w-28' },
		{ key: 'missionType', label: 'Mission type', class: 'whitespace-nowrap' },
		{ key: 'planet', label: 'Planet' },
		{ key: 'node', label: 'Node' },
		{ key: 'mode', label: 'Mode', class: 'whitespace-nowrap' },
		{ key: 'expiry', label: 'Expires in', align: 'right', class: 'w-32 whitespace-nowrap' },
	];

	const fissureFilterOptions = [
		{ value: 'normal', label: 'Normal' },
		{ value: 'steelPath', label: 'Steel Path' },
		{ value: 'voidStorm', label: 'Void Storms' },
	] as const;
	type FissureFilter = (typeof fissureFilterOptions)[number]['value'];

	let { world, now }: DashboardViewProps = $props();
	let fissures = $derived(world.fissures);
	let fissureFilter = $state<FissureFilter[]>(['normal', 'steelPath']);
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
				const category = fissure.isStorm ? 'voidStorm' : fissure.isHard ? 'steelPath' : 'normal';
				return fissureFilter.includes(category);
			})
			.filter((fissure) => eraFilter === 'all' || fissure.tier === eraFilter)
			.sort(
				(a, b) =>
					Number(a.tierNum) - Number(b.tierNum) ||
					(a.expiry?.getTime() ?? 0) - (b.expiry?.getTime() ?? 0),
			),
	);

	function eraLabelClass(tier: string) {
		const base = 'border px-2 py-1 font-medium text-sm';
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

<div class="@container/filters flex flex-col gap-2">
	<div class="flex flex-wrap @max-[52rem]/filters:justify-between items-center gap-x-8 gap-y-4">
		<div class="flex flex-col gap-1">
			<div class="font-semibold text-muted-foreground text-sm">Era</div>
			<RadioGroup label="Filter fissure era" options={eraFilterOptions} bind:value={eraFilter} />
		</div>
		<div class="flex flex-col gap-1">
			<div class="font-semibold text-muted-foreground text-sm">Mission category</div>
			<ToggleGroup
				label="Filter fissure mission type"
				options={fissureFilterOptions}
				bind:value={fissureFilter}
			/>
		</div>
	</div>
</div>

{#snippet fissureRow(fissure: DashboardViewProps['world']['fissures'][number])}
	{@const location = fissure.node.match(/^(.*) \(([^()]*)\)$/)}
	<tr class="hover:bg-surface/30 border-border-secondary border-t">
		<td class="p-3 whitespace-nowrap">
			<span class={eraLabelClass(fissure.tier)}>{fissure.tier}</span>
		</td>
		<td class="p-3">
			{fissure.missionType === 'Extermination' ? 'Exterminate' : fissure.missionType}
		</td>
		<td class="p-3 text-muted-foreground">{location?.[2] ?? '—'}</td>
		<td class="p-3 text-muted-foreground">{location?.[1] ?? fissure.node}</td>
		<td class="p-3 text-muted-foreground whitespace-nowrap">
			{#if fissure.isStorm}
				<span
					class="bg-indigo-400/20 px-2 py-1 border border-indigo-400/50 font-medium text-indigo-400 text-sm"
				>
					Void Storm
				</span>
			{/if}
			{#if fissure.isHard}
				<span
					class="bg-red-400/20 saturate-25 px-2 py-1 border border-red-400/50 font-medium text-red-400 text-sm"
				>
					Steel Path
				</span>
			{/if}
			{#if !fissure.isStorm && !fissure.isHard}
				<span
					class="bg-muted/20 px-2 py-1 border border-muted/50 font-medium text-muted-foreground text-sm"
				>
					Normal
				</span>
			{/if}
		</td>
		<td class="p-3 text-muted-foreground text-right whitespace-nowrap">
			{#if fissure.expiry}
				<time class="tabular-nums" datetime={fissure.expiry.toISOString()}>
					{formatTimeLeft(fissure.expiry, now)}
				</time>
			{/if}
		</td>
	</tr>
{/snippet}

<Table
	{columns}
	rows={activeFissures}
	rowKey={(fissure) =>
		fissure.id ?? `${fissure.nodeKey}:${fissure.tier}:${fissure.activation?.getTime()}`}
	renderRow={fissureRow}
	emptyMessage="No fissures match the selected filters."
/>

<style>
	.omnia-era {
		background: linear-gradient(105deg, #d0877070, #4c566a70, #d8dee970, #ebcb8b70);
	}
</style>
