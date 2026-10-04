<script lang="ts">
	import type { Snippet } from 'svelte';
	import { formatTimeLeft } from '$lib/date';

	let { children, action, footer, completed = false, expiry, now }: {
		children: Snippet;
		action?: Snippet;
		footer?: Snippet;
		completed?: boolean;
		expiry?: Date;
		now: number;
	} = $props();
	let hasExpiry = $derived(expiry instanceof Date && Number.isFinite(expiry.getTime()));
</script>

<!-- Render inside a list; each view supplies its content and controls. -->
<li class="border transition-colors {completed ? 'bg-surface/30 border-surface' : 'bg-background border-border-secondary'}">
	<div class="flex items-start justify-between gap-4 p-4">
		<div class="min-w-0">{@render children()}</div>
		{#if action}{@render action()}{/if}
	</div>
	{#if footer || hasExpiry}
		<div class="flex flex-wrap items-center justify-between gap-x-4 gap-y-2 border-t border-surface bg-surface/20 px-4 py-3">
			{#if footer}<div class="min-w-0">{@render footer()}</div>{/if}
			{#if hasExpiry && expiry}
				<span class="text-sm text-muted-foreground tabular-nums whitespace-nowrap">
					{#if expiry.getTime() > now}
						Ends in <time datetime={expiry.toISOString()} title={expiry.toLocaleString()}>{formatTimeLeft(expiry, now)}</time>
					{:else}Schedule updating{/if}
				</span>
			{/if}
		</div>
	{/if}
</li>
