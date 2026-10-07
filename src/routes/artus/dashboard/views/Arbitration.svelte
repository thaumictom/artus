<script lang="ts">
	import { openUrl } from '@tauri-apps/plugin-opener';
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import Table from '$lib/components/Table.svelte';
	import ToggleGroup from '$lib/components/ToggleGroup.svelte';
	import type { TableColumn } from '$lib/components/table-types';
	import scheduleText from '$lib/data/arbys.txt?raw';
	import nodeDetails from '$lib/data/arbitration-nodes.json';
	import ViewPanel from './ViewPanel.svelte';
	import { oracleBounties } from '$lib/oracle-bounties.svelte';
	import type { DashboardViewProps, ViewRow } from './view-types';

	type NodeKey = keyof typeof nodeDetails;
	type ScheduleEntry = { startsAt: number; nodeKey: NodeKey };
	type TableEntry = { kind: 'date'; startsAt: number } | { kind: 'mission'; entry: ScheduleEntry };
	const schedule: ScheduleEntry[] = scheduleText
		.trim()
		.split(/\r?\n/)
		.map((line) => {
			const [timestamp, nodeKey] = line.split(',');
			return { startsAt: Number(timestamp) * 1000, nodeKey: nodeKey as NodeKey };
		});
	const columns: TableColumn[] = [
		{ key: 'startsAt', label: 'Time', class: 'whitespace-nowrap w-24' },
		{ key: 'mission', label: 'Mission', class: 'whitespace-nowrap' },
		{ key: 'planet', label: 'Planet' },
		{ key: 'node', label: 'Node' },
		{ key: 'faction', label: 'Faction' },
		{ key: 'tier', label: 'Tier', class: 'w-16' },
	];
	const rankOptions = (['S', 'A', 'B', 'C', 'D', 'F'] as const).map((rank) => ({
		value: rank,
		label: rank,
	}));
	type Rank = (typeof rankOptions)[number]['value'];
	const dateFormat = new Intl.DateTimeFormat(undefined, {
		weekday: 'long',
		year: 'numeric',
		month: 'long',
		day: 'numeric',
	});
	const timeFormat = new Intl.DateTimeFormat(undefined, {
		hour: '2-digit',
		minute: '2-digit',
	});
	function daysUntil(timestamp: number, now: number): number {
		const date = new Date(timestamp);
		const today = new Date(now);
		const targetDay = Date.UTC(date.getFullYear(), date.getMonth(), date.getDate());
		const currentDay = Date.UTC(today.getFullYear(), today.getMonth(), today.getDate());
		return (targetDay - currentDay) / 86_400_000;
	}

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
	let selectedRanks = $state<Rank[]>(['S', 'A']);
	let upcoming = $derived(
		schedule
			.slice(firstAfter(currentHour * 3_600_000))
			.filter((entry) => selectedRanks.some((rank) => rank === nodeDetails[entry.nodeKey].tier)),
	);
	let tableRows = $derived.by(() => {
		const rows: TableEntry[] = [];
		let previousDate = '';
		for (const entry of upcoming) {
			const date = new Date(entry.startsAt).toDateString();
			if (date !== previousDate) {
				rows.push({ kind: 'date', startsAt: entry.startsAt });
				previousDate = date;
			}
			rows.push({ kind: 'mission', entry });
		}
		return rows;
	});
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
	let rows: ViewRow[] = $derived(
		current
			? [
					{
						title: current.node,
						description: [current.missionType, current.faction].filter(Boolean).join(' • '),
					},
				]
			: [],
	);
</script>

<ViewPanel
	title="Arbitration"
	{rows}
	now={localNow}
	headerSummary="Current Arbitration"
	expiry={current ? new Date(current.expiry) : undefined}
	empty={oracleBounties.loading
		? 'Loading current Arbitration…'
		: 'Arbitration is unavailable. Automatic retries are limited to once every five minutes.'}
>
	{#snippet headerAction()}
		<Button
			variant="link"
			size="none"
			href="https://browse.wf/arbys"
			class="inline-flex items-center gap-2 text-sm"
			onclick={(event) => {
				event.preventDefault();
				void openSchedule();
			}}
		>
			browse.wf <Icon icon="lucide:external-link" class="size-4" />
		</Button>
	{/snippet}
</ViewPanel>
{#if linkError}<p role="status" class="text-danger text-sm">
		Could not open the Arbitration schedule.
	</p>{/if}
<div class="flex flex-wrap justify-between gap-2 -mt-1 text-muted-foreground text-sm">
	<p>
		Source: <a
			href="https://browse.wf/arbys"
			class="hover:underline"
			onclick={(event) => {
				event.preventDefault();
				void openSchedule();
			}}
		>
			browse.wf
		</a>
		· MIT License © 2025 Calamity, Inc.
	</p>
	<p>Tier ratings by Arbitration Goons</p>
</div>
<hr class="my-6 border-surface" />

<section class="flex flex-col gap-3" aria-label="Upcoming Arbitrations">
	<h2 class="font-semibold text-lg">Upcoming Arbitrations</h2>
	<div class="flex flex-col gap-1">
		<div class="font-semibold text-muted-foreground text-sm">Rank</div>
		<ToggleGroup
			label="Filter Arbitration ranks"
			options={rankOptions}
			bind:value={selectedRanks}
		/>
	</div>
	{#snippet arbitrationRow(
		row: TableEntry,
		index: number,
		measureRow: (element: HTMLTableRowElement) => void,
	)}
		{#if row.kind === 'date'}
			{@const daysAway = daysUntil(row.startsAt, localNow)}
			<tr data-index={index} use:measureRow class="border-border-secondary border-t">
				<td
					colspan={columns.length}
					class="bg-surface/50 px-3 py-2 font-medium text-muted-foreground text-sm"
				>
					<time datetime={new Date(row.startsAt).toISOString()}>
						{dateFormat.format(row.startsAt)}
					</time>
					{#if daysAway > 0}<span class="font-normal">
							&nbsp;·&nbsp;&nbsp;in {daysAway}
							{daysAway === 1 ? 'day' : 'days'}
						</span>{/if}
				</td>
			</tr>
		{:else}
			{@const entry = row.entry}
			{@const detail = nodeDetails[entry.nodeKey]}
			<tr
				data-index={index}
				use:measureRow
				class="hover:bg-surface/30 border-border-secondary border-t"
			>
				<td class="p-3 whitespace-nowrap">
					<time
						datetime={new Date(entry.startsAt).toISOString()}
						title={new Date(entry.startsAt).toLocaleString()}
					>
						{timeFormat.format(entry.startsAt)}
					</time>
				</td>
				<td class="p-3">{detail.mission}</td>
				<td class="p-3 text-muted-foreground">{detail.planet}</td>
				<td class="p-3 text-muted-foreground">{detail.node}</td>
				<td class="p-3 text-muted-foreground">{detail.faction}</td>
				<td
					class="p-3 font-semibold"
					class:text-accent={detail.tier === 'S'}
					class:text-foreground={detail.tier === 'A'}
					class:text-muted-foreground={detail.tier !== 'S' && detail.tier !== 'A'}
				>
					{detail.tier}
				</td>
			</tr>
		{/if}
	{/snippet}
	<Table
		{columns}
		rows={tableRows}
		rowKey={(row) =>
			row.kind === 'date' ? `date-${row.startsAt}` : `mission-${row.entry.startsAt}`}
		renderRow={arbitrationRow}
		minWidth="760px"
		virtualize
		emptyMessage={selectedRanks.length === rankOptions.length
			? 'No more Arbitrations in the downloaded schedule.'
			: 'No upcoming Arbitrations match the selected ranks.'}
	/>
</section>
