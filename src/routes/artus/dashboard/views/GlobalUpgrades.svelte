<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.globalUpgrades ?? []).filter((item) => isCurrent(item, now)).map((item) => ({
		title: item.upgrade, description: item.desc, expiry: item.expiry,
	})));
</script>

<ViewPanel title="Global Boosters" {rows} {now} />

