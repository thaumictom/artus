<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, rewardText, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.events ?? []).filter((item) => isCurrent(item, now)).map((item) => ({
		title: item.description, description: [item.node, item.faction, item.tooltip].filter(Boolean).join(' · '),
		details: (item.rewards ?? []).map(rewardText), expiry: item.expiry,
		value: item.maximumScore > 0 ? `${item.currentScore.toLocaleString()} / ${item.maximumScore.toLocaleString()}` : undefined,
	})));
</script>

<ViewPanel title="Events" {rows} {now} />

