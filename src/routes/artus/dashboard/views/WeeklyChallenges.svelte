<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(isCurrent(world.weeklyChallenges, now) ? (world.weeklyChallenges?.challenges ?? []).map((item) => ({
		title: item.type, description: `${item.requiredAmount} required · Minimum enemy level ${item.minEnemyLevel}`,
		details: [item.damageType, item.target].filter((value): value is string => Boolean(value)), expiry: world.weeklyChallenges?.expiry,
	})) : []);
</script>

<ViewPanel title="Weekly Challenges" {rows} {now} />

