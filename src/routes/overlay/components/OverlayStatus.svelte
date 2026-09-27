<script lang="ts">
	import Icon from '@iconify/svelte';
	import { fade } from 'svelte/transition';
	import { flyAndScale } from '$lib/transition';

	let {
		relicFeedback,
		processing,
		listingStatus,
	}: {
		relicFeedback: string | null;
		processing: boolean;
		listingStatus: string | null;
	} = $props();
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
{#if processing || listingStatus}
	<div
		in:flyAndScale={{ y: 24 }}
		out:fade={{ duration: 100 }}
		class="absolute inset-0 flex justify-center items-center"
	>
		<div class="flex items-center gap-4 bg-background/90 p-4 border max-w-[calc(100vw-2rem)]">
			<Icon
				icon={listingStatus?.startsWith('Could not') ? 'lucide:circle-alert' : 'material-symbols:progress-activity'}
				class={`size-5 shrink-0 ${listingStatus?.startsWith('Could not') ? 'text-danger' : 'animate-spin'}`}
			/>
			<span class="text-foreground text-sm">{processing ? 'Processing…' : listingStatus}</span>
		</div>
	</div>
{/if}
