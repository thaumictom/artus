<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import Button from '$lib/components/Button.svelte';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import Icon from '@iconify/svelte';
	import Table from '$lib/components/Table.svelte';
	import WarframeItem from '$lib/components/WarframeItem.svelte';
	import type { TableColumn } from '$lib/components/table-types';
	import { warframeItems, initializeWarframeItems } from '$lib/warframe-item.svelte';
	import { formatTimeLeft } from '$lib/date';
	import {
		UTC_DAY,
		wikiSources,
		wikiRotation,
		wikiOfferingsSchema,
		type WikiSource,
		type WikiOfferings,
		type WikiOffering,
	} from '$lib/wiki-offerings';
	import type { DashboardViewProps } from './view-types';

	let { source, now }: Pick<DashboardViewProps, 'world' | 'now'> & { source: WikiSource } = $props();
	let data = $state<WikiOfferings | null>(null);
	let loading = $state(false);
	let error = $state('');
	let linkError = $state('');
	let mounted = $state(false);
	let request = 0;
	let requestedDay = -1;
	let currentSource: WikiSource | undefined;
	const config = $derived(wikiSources[source]);
	const rotation = $derived(wikiRotation(source, now));
	const sameRotation = $derived(data?.fetchedAt != null && data.fetchedAt >= rotation.start);
	const batchMatches = $derived(source !== 'coda' || data?.reportedBatch === rotation.batch);
	const reportCurrent = $derived(
		data?.observedAt != null && data.observedAt >= rotation.start && data.observedAt <= now,
	);
	const rows = $derived<WikiOffering[]>(
		source === 'coda' && (!batchMatches || !sameRotation)
			? (data?.batches[rotation.batch] ?? []).map((name) => ({ name, element: null, bonus: null }))
			: (data?.items ?? []),
	);
	const columns = $derived<TableColumn[]>(
		source === 'acrithis'
			? [{ key: 'name', label: 'Item' }]
			: [
					{ key: 'name', label: 'Weapon' },
					{ key: 'element', label: 'Element' },
					{ key: 'bonus', label: 'Bonus', align: 'right', class: 'w-24' },
					{ key: 'fusions', label: 'Max fusions', align: 'right', class: 'w-28 whitespace-nowrap' },
				],
	);
	const utc = (timestamp: number) =>
		new Date(timestamp).toLocaleString(undefined, { timeZone: 'UTC', timeZoneName: 'short' });
	const rotationDate = (timestamp: number) =>
		new Date(timestamp).toLocaleDateString('en-GB', {
			timeZone: 'UTC',
			day: '2-digit',
			month: '2-digit',
			year: 'numeric',
		});

	// Use the displayed minimum values, including the automatic round-up at 58%.
	const fusionThresholds = [58.0, 52.8, 48.0, 43.6, 39.7, 36.1, 32.8, 29.8, 27.1, 25.0];
	function maximumFusions(bonus: number | null) {
		if (bonus == null || !Number.isFinite(bonus) || bonus < 25 || bonus > 60) return null;
		return fusionThresholds.findIndex((minimum) => bonus >= minimum);
	}

	function bonusColor(bonus: number | null) {
		if (bonus === 60) return 'text-accent';
		if (bonus != null && bonus >= 52.8) return 'text-foreground';
		if (bonus != null && bonus >= 48) return 'text-elevated-foreground';
		return 'text-muted-foreground';
	}

	// Resolve wiki names against the shared live catalogs without inventing market slugs.
	function reference(name: string) {
		const normalized = name.replace(/^\d+\s+/, '').toLocaleLowerCase();
		const identity = Object.values(warframeItems.bySlug).find(
			(item) => item.name.toLocaleLowerCase() === normalized,
		);
		if (identity) return identity.gameRef ?? identity.slug;
		return (
			Object.entries(warframeItems.catalog).find(
				([, item]) => item.name?.toLocaleLowerCase() === normalized,
			)?.[0] ?? ''
		);
	}

	async function load() {
		const id = ++request;
		loading = true;
		error = '';
		try {
			const result = wikiOfferingsSchema.parse(await invoke('get_wiki_offerings', { source }));
			if (mounted && id === request) data = result;
		} catch (cause) {
			if (mounted && id === request) error = String(cause);
		} finally {
			if (mounted && id === request) loading = false;
		}
	}

	async function openLink(url: string) {
		try {
			await openUrl(url);
			linkError = '';
		} catch {
			linkError = 'Could not open the wiki link.';
		}
	}

	onMount(() => {
		const stop = initializeWarframeItems();
		mounted = true;
		return () => {
			mounted = false;
			request++;
			stop();
		};
	});
	$effect(() => {
		if (!mounted) return;
		const day = Math.floor(now / UTC_DAY);
		if (source !== currentSource || day !== requestedDay) {
			if (source !== currentSource) data = null;
			currentSource = source;
			requestedDay = day;
			void load();
		}
	});
</script>

{#snippet row(item: WikiOffering)}
	{@const quantity = item.name.match(/^(\d+)\s+/)?.[1]}
	<tr class="hover:bg-surface/70 border-border-secondary border-t transition-colors">
		<td class="px-3 py-3.5">
			<div class="flex items-start gap-2">
				{#if quantity}<span class="tabular-nums">{Number(quantity).toLocaleString()}×</span>{/if}
				<WarframeItem item={reference(item.name)} name={item.name.replace(/^\d+\s+/, '')} />
			</div>
		</td>
		{#if source !== 'acrithis'}
			<td class="px-3 py-3.5">{item.element ?? 'Not reported'}</td>
			<td class={`px-3 py-3.5 tabular-nums text-right ${bonusColor(item.bonus)}`}>
				{item.bonus != null ? `${item.bonus.toFixed(1)}%` : 'Not reported'}
			</td>
			<td class="px-3 py-3.5 tabular-nums text-right">
				{maximumFusions(item.bonus) ?? 'Not reported'}
			</td>
		{/if}
	</tr>
{/snippet}

<section class="flex flex-col gap-3 min-w-0" aria-label={config.title}>
	<div class="flex flex-wrap justify-between items-start gap-3">
		<div>
			<div class="flex items-center gap-2">
				<h2 class="font-medium text-xl">{config.title}</h2>
				<Tooltip
					side="right"
					align="start"
					class="text-muted-foreground hover:text-foreground"
					triggerProps={{ 'aria-label': `About ${config.title}` }}
				>
					<Icon icon="lucide:info" class="size-3.5" aria-hidden="true" />
					{#snippet content()}
						<div class="flex flex-col gap-2">
							{#if source === 'tenet'}
								<p>
									Ergo Glast offers weapons for 40 Corrupted Holokeys each. Offerings reset every
									four days at midnight UTC.
								</p>
							{:else if source === 'coda'}
								<p>
									Eleanor offers weapons for 10 Live Heartcells each. Batches A and B alternate
									every four days at midnight UTC.
								</p>
							{:else}
								<p>Acrithis offerings reset weekly on Monday at midnight UTC.</p>
							{/if}
							<p>
								Offerings and bonuses are player-reported and may be incomplete or inaccurate.
								Weapon reports have no observation date, so their freshness is unconfirmed.
							</p>
							{#if source !== 'acrithis'}
								<p>Max fusions shows the maximum number needed to reach 60.0% from the reported bonus. Values of 58.0% or higher automatically round up to 60.0%.</p>
							{/if}
							{#if data?.fetchedAt != null}<p>Last fetched: {utc(data.fetchedAt)}</p>{/if}
							{#if data?.pageUpdatedAt != null}<p>
									Wiki page last edited: {utc(data.pageUpdatedAt)} (may include unrelated edits).
								</p>{/if}
							<p>Each page is requested at most once per day and cached.</p>
						</div>
					{/snippet}
				</Tooltip>
			</div>
			<p class="text-muted-foreground text-base">
				{source === 'tenet'
					? 'Ergo Glast'
					: source === 'coda'
						? `Eleanor · Batch ${rotation.batch}`
						: 'Weekly offerings'} ·
				<a
					href={config.url}
					class="text-accent hover:underline"
					onclick={(event) => {
						event.preventDefault();
						void openLink(config.url);
					}}
				>
					View on WARFRAME Wiki ↗
				</a>
			</p>
		</div>
		<div class="text-base text-right">
			<p>Resets in {formatTimeLeft(new Date(rotation.end), now)}</p>
			<p class="text-muted-foreground">
				<time datetime={new Date(rotation.start).toISOString()}>
					{rotationDate(rotation.start)}
				</time>
				-
				<time datetime={new Date(rotation.end).toISOString()}>{rotationDate(rotation.end)}</time>
			</p>
		</div>
	</div>
	{#if loading || (data?.fetchedAt != null && (!sameRotation || !batchMatches || source === 'acrithis')) || data?.observedAt != null}
		<div class="text-muted-foreground text-base" role="status">
			{#if loading}<p>Reading wiki offerings…</p>{/if}
			{#if data?.fetchedAt != null}
				{#if !sameRotation || !batchMatches || source === 'acrithis'}<p>
						{#if !sameRotation || !batchMatches}Reports are from a previous rotation. They may not
							match the current offerings.
						{:else if source === 'acrithis'}{reportCurrent
								? 'Report is up to date for this rotation.'
								: 'Report is outdated or its observation date is unavailable.'}
						{/if}
					</p>{/if}
			{/if}
			{#if data?.observedAt != null}<p>Last reported: {utc(data.observedAt)}</p>{/if}
		</div>
	{/if}
	{#if error || data?.error}<p class="text-danger text-base" role="alert">
			{error || data?.error}
		</p>{/if}
	{#if error}<Button onclick={load} disabled={loading}>Read cached status</Button>{/if}
	{#if linkError}<p class="text-danger text-base" role="alert">{linkError}</p>{/if}
	{#if data?.items.length && (source === 'acrithis' ? !reportCurrent : !sameRotation || !batchMatches)}
		<p class="text-muted-foreground text-base">
			{source === 'coda'
				? 'Current batch weapons; bonuses have not been reported for this rotation.'
				: 'Previous reports shown below; current offerings are unconfirmed.'}
		</p>
	{/if}
	<Table
		{columns}
		{rows}
		rowKey={(item) => item.name}
		renderRow={row}
		minWidth={source === 'acrithis' ? '280px' : '540px'}
		emptyMessage={loading ? 'Loading offerings…' : 'No reported offerings available.'}
	/>
	<div class="flex flex-wrap justify-between gap-2 text-muted-foreground text-sm">
		<p>
			Source: WARFRAME Wiki contributors · <a
				href="https://creativecommons.org/licenses/by-nc-sa/3.0/"
				class="hover:underline"
				onclick={(event) => {
					event.preventDefault();
					void openLink(event.currentTarget.href);
				}}
			>
				CC BY-NC-SA 3.0
			</a>
		</p>
		<p>Information may be inaccurate.</p>
	</div>
</section>
