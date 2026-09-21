<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { isCurrent, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived(isCurrent(world.descendia, now) ? (world.descendia?.challenges ?? []).map((item) => ({
		title: `${item.index + 1}. ${item.type}`, description: [item.challenge, item.level].filter(Boolean).join(' · '),
		details: [...item.specs.map((spec) => spec.name), ...item.auras.map((aura) => aura.name)], expiry: world.descendia?.expiry,
	})) : []);
</script>

<ViewPanel title="Descendia" {rows} {now} />

