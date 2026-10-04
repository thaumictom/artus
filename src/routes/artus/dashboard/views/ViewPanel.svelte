<script lang="ts">
	import type { Snippet } from 'svelte';
	import { formatTimeLeft } from '$lib/date';
	import { validDate, type ViewRow } from './view-types';
	let { title, rows, now, summary, rowAction, headerAction, empty = 'No active entries in this snapshot.' }: {
		title: string;
		rows: ViewRow[];
		now: number;
		summary?: string;
		rowAction?: Snippet<[ViewRow]>;
		headerAction?: Snippet;
		empty?: string;
	} = $props();
</script>

<section class="bg-background border border-surface min-w-0" aria-label={title}>
	<header class="p-4 border-b border-surface">
		<div class="flex flex-wrap items-center justify-between gap-2">
			<h2 class="font-medium">{title}</h2>
			{#if headerAction}{@render headerAction()}{/if}
		</div>
		{#if summary}<p class="mt-1 text-muted-foreground text-base">{summary}</p>{/if}
	</header>
	<ul class="divide-y divide-surface">
		{#each rows as row}
			<li class="p-4 text-base">
				<div class="flex flex-wrap justify-between items-baseline gap-2">
					<h3 class="font-medium break-words min-w-0">{row.title}</h3>
					{#if row.value}<span class="text-accent tabular-nums">{row.value}</span>{/if}
					{#if rowAction}{@render rowAction(row)}{/if}
				</div>
				{#if row.description}<p class="mt-1 text-muted-foreground whitespace-pre-line break-words">{row.description}</p>{/if}
				{#each row.details ?? [] as detail}<p class="mt-1 text-muted-foreground break-words">{detail}</p>{/each}
				{#if validDate(row.expiry)}
					<p class="mt-2 text-muted-foreground text-sm tabular-nums">
						{#if row.expiry.getTime() > now}Ends in <time datetime={row.expiry.toISOString()}>{formatTimeLeft(row.expiry, now)}</time>{:else}Schedule updating{/if}
					</p>
				{/if}
			</li>
		{:else}<li class="p-4 text-muted-foreground text-base">{empty}</li>{/each}
	</ul>
</section>
