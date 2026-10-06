<script lang="ts">
	import { bountyRotationRewards, type BountyRotation, type BountyRotationKind } from '$lib/bounty-rotation-rewards';
	import { warframeItems } from '$lib/warframe-item.svelte';
	import WarframeItem from './WarframeItem.svelte';

	let { kind, rotation }: { kind: BountyRotationKind; rotation: BountyRotation } = $props();
	let rewards = $derived(bountyRotationRewards[kind][rotation].map((reward) => ({
		...reward,
		// Untradeable Caliban parts have no market slug; resolve them from the shared live catalog.
		reference: reward.slug ?? Object.entries(warframeItems.catalog).find(([, item]) =>
			item.name === reward.name || item.name === `${reward.name} Blueprint`)?.[0] ?? '',
	})));
</script>

<div class="flex flex-col gap-2 text-sm">
	<p class="text-muted-foreground">{kind === 'cetus' ? 'Cetus bounty' : 'Isolation Vault'} rotation {rotation} · Notable rewards</p>
	<div class="flex flex-wrap gap-x-6 gap-y-2">
		{#each rewards as reward (reward.name)}
			<WarframeItem item={reward.reference} name={reward.name} tradable={reward.slug ? undefined : false} />
		{/each}
	</div>
</div>
