<script lang="ts">
	import { formatTimeLeft } from '$lib/date';
	import Button from '$lib/components/Button.svelte';
	import { wikiRotation, wikiSources } from '$lib/wiki-offerings';

	let { now, onOpen }: {
		now: number;
		onOpen: (source: 'tenet' | 'coda') => void;
	} = $props();
	const sources = ['tenet', 'coda'] as const;
</script>

<section aria-label="Weapon resets" class="grid grid-cols-1 gap-2 sm:grid-cols-2">
	{#each sources as source (source)}
		{@const reset = new Date(wikiRotation(source, now).end)}
		<Button
			size="small"
			class="flex items-baseline justify-between gap-2 border-surface bg-background min-w-0 text-left focus-visible:outline-2 focus-visible:outline-accent"
			onclick={() => onOpen(source)}
		>
			<span class="text-base">{wikiSources[source].title}</span>
			<span class="text-muted-foreground text-sm tabular-nums whitespace-nowrap">
				Reset in <time datetime={reset.toISOString()}>{formatTimeLeft(reset, now)}</time>
			</span>
		</Button>
	{/each}
</section>
