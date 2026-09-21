<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, amount, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.conclaveChallenges ?? []).filter((item) => isCurrent(item, now)).map((item) => ({
		title: item.title || item.mode, description: item.description,
		details: [`${item.mode} · ${item.daily ? 'Daily' : 'Weekly'} · ${item.amount} required`],
		value: item.standing !== undefined ? amount(item.standing, 'standing') : undefined, expiry: item.expiry,
	})));
</script>

<ViewPanel title="Conclave Challenges" {rows} {now} />

