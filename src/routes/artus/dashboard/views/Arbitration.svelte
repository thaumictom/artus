<script lang="ts">
	import { openUrl } from '@tauri-apps/plugin-opener';
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import ViewPanel from './ViewPanel.svelte';
	import { oracleBounties } from '$lib/oracle-bounties.svelte';
	import type { DashboardViewProps, ViewRow } from './view-types';
	let { localNow }: DashboardViewProps = $props();
	let linkError = $state(false);
	async function openSchedule() {
		try {
			await openUrl('https://browse.wf/arbys');
			linkError = false;
		} catch {
			linkError = true;
		}
	}
	let arbitration = $derived(oracleBounties.snapshot?.arbitration);
	let current = $derived(arbitration && arbitration.expiry > localNow ? arbitration : null);
	let rows: ViewRow[] = $derived(current ? [{ title: current.node, description: [current.missionType, current.faction].filter(Boolean).join(' · ') }] : []);
</script>

<ViewPanel title="Arbitration" {rows} now={localNow} headerSummary="Current Arbitration"
	expiry={current ? new Date(current.expiry) : undefined}
	empty={oracleBounties.loading ? 'Loading current Arbitration…' : 'Arbitration is unavailable. Automatic retries are limited to once every five minutes.'}>
	{#snippet headerAction()}
		<Button variant="link" size="none" href="https://browse.wf/arbys" class="inline-flex items-center gap-2 text-sm"
			onclick={(event) => { event.preventDefault(); void openSchedule(); }}>
			browse.wf <Icon icon="lucide:external-link" class="size-4" />
		</Button>
	{/snippet}
</ViewPanel>
{#if linkError}<p role="status" class="mt-3 text-sm text-danger">Could not open the Arbitration schedule.</p>{/if}
