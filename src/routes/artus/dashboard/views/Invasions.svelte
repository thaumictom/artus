<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { rewardText, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.invasions ?? []).filter((item) => !item.completed).map((item) => ({
		title: item.node, description: item.desc,
		details: [
			`${item.attacker.faction}: ${rewardText(item.attacker.reward)}`,
			`${item.defender.faction}: ${rewardText(item.defender.reward)}`,
		],
		value: Number.isFinite(item.completion) ? `${Math.min(100, Math.max(0, item.completion)).toFixed(1)}% complete` : undefined,
	})));
</script>

<ViewPanel title="Invasions" {rows} {now} />

