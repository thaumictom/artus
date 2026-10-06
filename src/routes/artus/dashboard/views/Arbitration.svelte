<script lang="ts">
	import { openUrl } from '@tauri-apps/plugin-opener';
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import Table from '$lib/components/Table.svelte';
	import type { TableColumn } from '$lib/components/table-types';
	import scheduleText from '$lib/data/arbys.txt?raw';
	import nodeDetails from '$lib/data/arbitration-nodes.json';
	import ViewPanel from './ViewPanel.svelte';
	import { oracleBounties } from '$lib/oracle-bounties.svelte';
	import type { DashboardViewProps, ViewRow } from './view-types';

	type NodeKey = keyof typeof nodeDetails;
	type ScheduleEntry = { startsAt: number; nodeKey: NodeKey };
	const schedule: ScheduleEntry[] = scheduleText.trim().split(/\r?\n/).map((line) => {
		const [timestamp, nodeKey] = line.split(',');
		return { startsAt: Number(timestamp) * 1000, nodeKey: nodeKey as NodeKey };
	});
	const columns: TableColumn[] = [
		{ key: 'startsAt', label: 'Starts', class: 'whitespace-nowrap w-48' },
		{ key: 'mission', label: 'Mission', class: 'whitespace-nowrap' },
		{ key: 'planet', label: 'Planet' },
		{ key: 'node', label: 'Node' },
		{ key: 'faction', label: 'Faction' },
		{ key: 'tier', label: 'Tier', class: 'w-16' },
	];
	const startsFormat = new Intl.DateTimeFormat(undefined, {
		weekday: 'short', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
	});

	function firstAfter(timestamp: number): number {
		let low = 0;
		let high = schedule.length;
		while (low < high) {
			const middle = (low + high) >>> 1;
			if (schedule[middle].startsAt <= timestamp) low = middle + 1;
			else high = middle;
		}
		return low;
	}

	let { localNow }: DashboardViewProps = $props();
	let currentHour = $derived(Math.floor(localNow / 3_600_000));
	let upcoming = $derived(schedule.slice(firstAfter(currentHour * 3_600_000)));
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

<p class="text-muted-foreground text-xs">
	Schedule and tiers from browse.wf, MIT License © 2025 Calamity, Inc. Tier ratings credit the Arbitration Goons.
</p>
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

<section class="mt-6 flex flex-col gap-3" aria-label="Upcoming Arbitrations">
	<h2 class="text-lg font-semibold">Upcoming Arbitrations</h2>
	{#snippet arbitrationRow(entry: ScheduleEntry, index: number, measureRow: (element: HTMLTableRowElement) => void)}
		{@const detail = nodeDetails[entry.nodeKey]}
		<tr data-index={index} use:measureRow class="hover:bg-surface/30 border-border-secondary border-t">
			<td class="p-3 whitespace-nowrap"><time datetime={new Date(entry.startsAt).toISOString()} title={new Date(entry.startsAt).toLocaleString()}>{startsFormat.format(entry.startsAt)}</time></td>
			<td class="p-3">{detail.mission}</td>
			<td class="p-3 text-muted-foreground">{detail.planet}</td>
			<td class="p-3 text-muted-foreground">{detail.node}</td>
			<td class="p-3 text-muted-foreground">{detail.faction}</td>
			<td class="p-3 font-semibold">{detail.tier}</td>
		</tr>
	{/snippet}
	<Table {columns} rows={upcoming} rowKey={(entry) => String(entry.startsAt)} renderRow={arbitrationRow}
		minWidth="760px" virtualize emptyMessage="No more Arbitrations in the downloaded schedule." />
</section>
