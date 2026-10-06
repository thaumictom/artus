<script lang="ts">
	import WorldStateLabel from '$lib/components/WorldStateLabel.svelte';
	import { worldStateLabel } from '$lib/worldstate-labels';
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows = $derived(isCurrent(world.descendia, now) ? (world.descendia?.challenges ?? [])
		.toSorted((a, b) => a.index - b.index).map((item) => {
			const objective = worldStateLabel('descendiaObjective', item.typeKey, 'Unknown objective');
			const challenge = worldStateLabel('descendiaChallenge', item.challengeKey);
			const penances = [...new Set(item.auras.map((aura) => worldStateLabel('descendiaPenance', aura.uniqueName)).filter(Boolean))];
			return {
				title: `Floor ${item.index} · ${objective}`,
				index: item.index,
				objectiveKey: item.typeKey,
				description: [challenge !== objective && !penances.includes(challenge) ? challenge : '',
					worldStateLabel('descendiaLevel', item.levelUniqueName)].filter(Boolean).join(' · '),
				enemies: [...new Set(item.specs.map((spec) => worldStateLabel('descendiaEnemies', spec.uniqueName)).filter(Boolean))],
				penances,
			};
		}) : []);
</script>

<ViewPanel title="Descendia" countLabel="floors" {rows} {now} expiry={world.descendia?.expiry}
	rowFooterVisible={(row) => row.enemies.length > 0 || row.penances.length > 0}>
	{#snippet rowTitle(row)}
		<h3 class="font-semibold break-words">Floor {row.index} · <WorldStateLabel category="descendiaObjective" code={row.objectiveKey} fallback="Unknown objective" /></h3>
	{/snippet}
	{#snippet rowFooter(row)}
		{#if row.enemies.length || row.penances.length}
			<div class="flex flex-col gap-2 text-sm text-muted-foreground">
				{#if row.enemies.length}<p><span class="font-medium text-foreground">Enemies:</span> {row.enemies.join(' · ')}</p>{/if}
				{#if row.penances.length}<p><span class="font-medium text-foreground">Penances:</span> {row.penances.join(' · ')}</p>{/if}
			</div>
		{/if}
	{/snippet}
</ViewPanel>
