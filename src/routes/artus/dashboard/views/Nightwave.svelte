<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, amount, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let challenges = $derived((world.nightwave?.activeChallenges ?? []).filter((item) => isCurrent(item, now)));
	let stats = $derived([
		{ value: challenges.length, label: 'challenges' },
		{ value: challenges.filter((item) => item.isDaily).length, label: 'daily' },
		{ value: challenges.filter((item) => !item.isDaily && !item.isElite).length, label: 'weekly' },
		{ value: challenges.filter((item) => !item.isDaily && item.isElite).length, label: 'elite weekly' },
	]);
	let rows: ViewRow[] = $derived(challenges.map((item) => ({
		title: item.title, description: item.desc,
		details: [item.isDaily ? 'Daily' : item.isElite ? 'Elite weekly' : 'Weekly', ...(item.isPermanent ? ['Permanent challenge'] : [])],
		value: amount(item.reputation, 'standing'), expiry: item.expiry,
	})));
</script>

<ViewPanel title="Nightwave" {stats} {rows} {now} />
