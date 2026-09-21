<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { amount, type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.darkSectors ?? []).map((item) => ({
		title: item.mission?.node || item.defenderName, description: [item.defenderName, item.deployerClan, item.defenderMOTD].filter(Boolean).join(' · '),
		details: [`Credit tax: ${item.creditTaxRate}% · Item tax: ${item.itemsTaxRate}%`],
		value: amount(item.perMissionBattlePay, 'credits per mission'),
	})));
</script>

<ViewPanel title="Dark Sectors" {rows} {now} />

