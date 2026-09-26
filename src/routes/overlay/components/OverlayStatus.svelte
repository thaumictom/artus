<script lang="ts">
	import Icon from '@iconify/svelte';
	import { fade } from 'svelte/transition';
	import { flyAndScale } from '$lib/transition';

	let { relicFeedback, processing }: { relicFeedback: string | null; processing: boolean } = $props();
</script>

{#if relicFeedback}
	<div class="bottom-4 left-1/2 absolute -translate-x-1/2">
		<div
			in:flyAndScale={{ y: 28, duration: 400 }}
			out:fade={{ duration: 450 }}
			class="bg-background/95 shadow-lg px-4 py-2 border border-accent text-foreground text-sm whitespace-nowrap"
		>
			{relicFeedback}
		</div>
	</div>
{/if}
{#if processing}
	<div
		in:flyAndScale={{ y: 24 }}
		out:fade={{ duration: 100 }}
		class="absolute inset-0 flex justify-center items-center"
	>
		<div class="flex items-center gap-4 bg-background/90 p-4 border">
			<Icon icon="material-symbols:progress-activity" class="size-5 animate-spin" />
			<span class="text-foreground text-sm">Processing…</span>
		</div>
	</div>
{/if}
