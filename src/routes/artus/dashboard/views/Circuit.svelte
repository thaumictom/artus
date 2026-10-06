<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import WarframeItem from '$lib/components/WarframeItem.svelte';
	import { warframeItems } from '$lib/warframe-item.svelte';
	import { type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.duviriCycle?.choices ?? []).map((choice) => ({
		title: choice.category === 'hard' ? 'Steel Path' : 'Normal', details: choice.choices,
	})));
	// Choices arrive as display names; resolve them against the shared live catalogs.
	let referencesByName = $derived.by(() => {
		const references = new Map<string, string>();
		for (const [reference, item] of Object.entries(warframeItems.catalog)) {
			if (item.name) references.set(item.name.trim().toLowerCase(), reference);
		}
		for (const item of Object.values(warframeItems.bySlug)) {
			references.set(item.name.trim().toLowerCase(), item.gameRef || item.slug);
		}
		return references;
	});
	let stats = $derived([
		{ value: rows.length, label: 'paths' },
		{ value: rows.reduce((total, row) => total + (row.details?.length ?? 0), 0), label: 'reward choices' },
	]);
</script>

<ViewPanel title="The Circuit" {rows} {now} {stats}>
	{#snippet rowContent(row)}
		<ul class="mt-4 flex flex-col gap-3">
			{#each row.details ?? [] as choice}
				{@const reference = choice.startsWith('/') ? choice : referencesByName.get(choice.trim().toLowerCase())}
				<li class="border-t border-surface pt-3">
					<WarframeItem item={reference ?? ''} name={choice} />
				</li>
			{/each}
		</ul>
	{/snippet}
	{#snippet rowFooter(row)}
		<p class="text-muted-foreground text-sm">{row.details?.length ?? 0} reward choices</p>
	{/snippet}
</ViewPanel>
