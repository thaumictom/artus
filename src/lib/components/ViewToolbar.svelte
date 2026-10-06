<script lang="ts">
	import type { Snippet } from 'svelte';

	let { stats = [], summary, loading = false, loadingText = 'Loading…', actions }: {
		stats?: { value: number; label: string }[];
		summary?: Snippet;
		loading?: boolean;
		loadingText?: string;
		actions?: Snippet;
	} = $props();
</script>

<div class="flex flex-col gap-4">
	<header class="flex flex-wrap justify-between items-center gap-4 w-full">
		<div class="flex flex-wrap items-center divide-border-secondary divide-x text-base tabular-nums" aria-live="polite">
			{#if summary}
				{@render summary()}
			{:else if loading}
				<p class="text-muted-foreground">{loadingText}</p>
			{:else}
				{#each stats as stat, index}
					<div class="px-4 first:pl-0 last:pr-0" class:text-muted-foreground={index > 0}>
						{stat.value} {stat.label}
					</div>
				{/each}
			{/if}
		</div>
		{#if actions}<div class="flex flex-wrap items-center gap-2">{@render actions()}</div>{/if}
	</header>
	<div class="bg-surface my-1 w-full h-px"></div>
</div>
