<script lang="ts">
	import { formatTimeLeft } from '$lib/date';
	import Button from '$lib/components/Button.svelte';
	import { wikiRotation, wikiSources } from '$lib/wiki-offerings';

	let {
		now,
		onOpen,
	}: {
		now: number;
		onOpen: (source: 'tenet' | 'coda') => void;
	} = $props();
	const sources = ['tenet', 'coda'] as const;
</script>

<section aria-label="Weapon resets" class="gap-2 grid grid-cols-1 sm:grid-cols-2">
	{#each sources as source (source)}
		{@const reset = new Date(wikiRotation(source, now).end)}
		<Button
			size="small"
			class="flex justify-between items-baseline gap-2 bg-background border-surface focus-visible:outline-2 focus-visible:outline-accent min-w-0 text-left"
			onclick={() => onOpen(source)}
		>
			<span class="text-base truncate">{wikiSources[source].title}</span>
			<span class="tabular-nums text-muted-foreground text-sm whitespace-nowrap">
				Reset in <time datetime={reset.toISOString()}>{formatTimeLeft(reset, now)}</time>
			</span>
		</Button>
	{/each}
</section>
