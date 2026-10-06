<script lang="ts" generics="T extends ViewRow">
	import type { Snippet } from 'svelte';
	import ViewCard from '$lib/components/ViewCard.svelte';
	import ViewToolbar from '$lib/components/ViewToolbar.svelte';
	import { formatTimeLeft } from '$lib/date';
	import { validDate, type ViewRow } from './view-types';

	let {
		title, rows, now, summary, header, headerSummary, expiry, rowAction, headerAction, rowContent, rowFooter, rowFooterVisible, rowTitle,
		countLabel = 'entries', stats, empty = 'No active entries in this snapshot.',
	}: {
		title: string;
		rows: T[];
		now: number;
		summary?: string;
		header?: Snippet;
		headerSummary?: string;
		expiry?: Date;
		rowAction?: Snippet<[T]>;
		headerAction?: Snippet;
		rowContent?: Snippet<[T]>;
		rowFooter?: Snippet<[T]>;
		rowFooterVisible?: (row: T) => boolean;
		rowTitle?: Snippet<[T]>;
		countLabel?: string;
		stats?: { value: number; label: string }[];
		empty?: string;
	} = $props();
</script>

<section class="flex flex-col gap-4 min-w-0" aria-label={title}>
	{#snippet toolbarSummary()}
		<p class="font-medium break-words">{headerSummary}</p>
	{/snippet}
	{#snippet toolbarActions()}
		{#if validDate(expiry)}
			<span class="text-sm text-muted-foreground tabular-nums whitespace-nowrap">
				{#if expiry.getTime() > now}
					Ends in <time datetime={expiry.toISOString()} title={expiry.toLocaleString()}>{formatTimeLeft(expiry, now)}</time>
				{:else}Schedule updating{/if}
			</span>
		{/if}
		{#if headerAction}{@render headerAction()}{/if}
	{/snippet}
	{#if header}
		{@render header()}
	{:else}
		<ViewToolbar
			stats={stats ?? [{ value: rows.length, label: countLabel }]}
			summary={headerSummary ? toolbarSummary : undefined}
			actions={headerAction || validDate(expiry) ? toolbarActions : undefined}
		/>
	{/if}
	{#if summary}<p class="text-muted-foreground text-sm break-words">{summary}</p>{/if}
	<ul class="flex flex-col gap-3">
		{#each rows as row}
			{#snippet details()}
				{#if rowFooter}
					{@render rowFooter(row)}
				{:else}
					<div class="flex flex-col gap-1 text-muted-foreground text-sm">
						{#each row.details ?? [] as detail}
							<p class="whitespace-pre-line break-words">{detail}</p>
						{/each}
					</div>
				{/if}
			{/snippet}
			<ViewCard {now} expiry={row.expiry} footer={(rowFooter ? (rowFooterVisible?.(row) ?? true) : row.details?.length) ? details : undefined}>
				{#if rowTitle}
					{@render rowTitle(row)}
				{:else}
					<h3 class="font-semibold break-words">{row.title}</h3>
				{/if}
				{#if row.description}
					<p class="mt-2 text-muted-foreground text-sm whitespace-pre-line break-words">{row.description}</p>
				{/if}
				{#if rowContent}{@render rowContent(row)}{/if}
				{#snippet action()}
					<div class="flex flex-wrap justify-end items-center gap-2">
						{#if row.value}
							<span class="inline-flex items-center rounded-full border border-border-secondary bg-surface/30 px-2.5 py-1 text-sm font-medium text-muted-foreground tabular-nums whitespace-nowrap">{row.value}</span>
						{/if}
						{#if rowAction}{@render rowAction(row)}{/if}
					</div>
				{/snippet}
			</ViewCard>
		{:else}
			<li class="p-6 border border-surface text-muted-foreground text-sm text-center">{empty}</li>
		{/each}
	</ul>
</section>
