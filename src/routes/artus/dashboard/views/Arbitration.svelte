<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { oracleBounties } from '$lib/oracle-bounties.svelte';
	import type { DashboardViewProps, ViewRow } from './view-types';
	let { localNow }: DashboardViewProps = $props();
	let arbitration = $derived(oracleBounties.snapshot?.arbitration);
	let current = $derived(arbitration && arbitration.expiry > localNow ? arbitration : null);
	let rows: ViewRow[] = $derived(current ? [{ title: current.node, description: [current.missionType, current.faction].filter(Boolean).join(' · ') }] : []);
</script>

<ViewPanel title="Arbitration" {rows} now={localNow} headerSummary="Current Arbitration"
	expiry={current ? new Date(current.expiry) : undefined}
	empty={oracleBounties.loading ? 'Loading current Arbitration…' : 'Arbitration is unavailable. Automatic retries are limited to once every five minutes.'} />
