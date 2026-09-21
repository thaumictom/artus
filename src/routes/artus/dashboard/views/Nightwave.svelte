<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, amount, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.nightwave?.activeChallenges ?? []).filter((item) => isCurrent(item, now)).map((item) => ({
		title: item.title, description: item.desc,
		details: [item.isDaily ? 'Daily' : item.isElite ? 'Elite weekly' : 'Weekly', ...(item.isPermanent ? ['Permanent challenge'] : [])],
		value: amount(item.reputation, 'standing'), expiry: item.expiry,
	})));
</script>

<ViewPanel title="Nightwave" {rows} {now} />

