<script lang="ts">
	import Currency from '$lib/components/Currency.svelte';
	import ViewPanel from './ViewPanel.svelte';
	import type { DashboardViewProps } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows = $derived((world.darkSectors ?? []).map((item) => ({
		title: item.mission?.node || item.defenderName, description: [item.defenderName, item.deployerClan, item.defenderMOTD].filter(Boolean).join(' · '),
		details: [`Credit tax: ${item.creditTaxRate}% · Item tax: ${item.itemsTaxRate}%`],
		battlePay: item.perMissionBattlePay,
	})));
</script>

<ViewPanel title="Dark Sectors" {rows} {now}>
	{#snippet rowAction(row)}
		<span class="inline-flex items-center gap-1 rounded-full border border-border-secondary bg-surface/30 px-2.5 py-1 text-sm font-medium text-muted-foreground">
			<Currency value={row.battlePay} currency="credits" /> per mission
		</span>
	{/snippet}
</ViewPanel>
