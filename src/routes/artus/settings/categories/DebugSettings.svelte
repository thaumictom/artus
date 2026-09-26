<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { platform } from '@tauri-apps/plugin-os';
	import { onMount } from 'svelte';
	import { ocrDebug } from '$lib/ocr-debug.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import DictionarySettings from '../components/DictionarySettings.svelte';
	import OcrBoundingBoxesSetting from '../components/OcrBoundingBoxesSetting.svelte';
	import OcrGroupingSettings from '../components/OcrGroupingSettings.svelte';

	type RelicSelectionImage = {
		png_bytes: number[];
		width: number;
		height: number;
		selected_slot: number | null;
		cluster_x: number | null;
		matched_item: string | null;
		reward_count: number;
		ocr_word_count: number;
		reward_edges: [string, number][];
		status: string;
	};
	const isWindows = platform() === 'windows';
	let relicImageUrl = $state<string | null>(null);
	let relicImageSize = $state<{ width: number; height: number } | null>(null);
	let relicSelectedSlot = $state<number | null>(null);
	let relicClusterX = $state<number | null>(null);
	let relicMatchedItem = $state<string | null>(null);
	let relicRewardCount = $state(0);
	let relicOcrWordCount = $state(0);
	let relicRewardEdges = $state<[string, number][]>([]);
	let relicStatus = $state('');

	onMount(() => {
		if (!isWindows) return;
		let disposed = false;
		let requestId = 0;
		let unlisten: (() => void) | undefined;
		async function refreshRelicImage() {
			const request = ++requestId;
			try {
				const image = await invoke<RelicSelectionImage | null>('get_relic_selection_debug_image');
				if (disposed || request !== requestId || !image) return;
				const nextUrl = URL.createObjectURL(new Blob([new Uint8Array(image.png_bytes)], { type: 'image/png' }));
				if (relicImageUrl) URL.revokeObjectURL(relicImageUrl);
				relicImageUrl = nextUrl;
				relicImageSize = { width: image.width, height: image.height };
				relicSelectedSlot = image.selected_slot;
				relicClusterX = image.cluster_x;
				relicMatchedItem = image.matched_item;
				relicRewardCount = image.reward_count;
				relicOcrWordCount = image.ocr_word_count;
				relicRewardEdges = image.reward_edges;
				relicStatus = image.status;
			} catch (error) {
				console.error('Could not load relic selection image:', error);
			}
		}
		void refreshRelicImage();
		void listen('relic_selection_capture_ready', refreshRelicImage).then((cleanup) => {
			if (disposed) cleanup();
			else unlisten = cleanup;
		});
		return () => {
			disposed = true;
			unlisten?.();
			if (relicImageUrl) URL.revokeObjectURL(relicImageUrl);
		};
	});
</script>

{#snippet imageTrigger()}
	<button class="block w-full cursor-zoom-in" aria-label="Enlarge OCR image">
		<img
			src={ocrDebug.imageUrl ?? ''}
			alt="Latest black and white OCR input"
			class="w-full rounded border border-border [image-rendering:pixelated]"
		/>
	</button>
{/snippet}

{#snippet imageTitle()}OCR image{/snippet}
{#snippet imageDescription()}Scroll to inspect the full-size image.{/snippet}
{#snippet imageClose()}<Button>Close</Button>{/snippet}

{#snippet relicImageTrigger()}
	<button class="block w-full cursor-zoom-in" aria-label="Enlarge relic selection capture">
		<span class="relative block">
			<img
				src={relicImageUrl ?? ''}
				alt="Last binary filtered relic selection strip"
				class="w-full border border-border [image-rendering:pixelated]"
			/>
			{#each relicRewardEdges as [, edge]}
				<span class="pointer-events-none absolute top-0 bottom-0 border-l border-amber-500" style:left={`${edge / (relicImageSize?.width ?? 1) * 100}%`}></span>
			{/each}
		</span>
	</button>
{/snippet}

{#snippet relicImageTitle()}Relic selection capture{/snippet}
{#snippet relicImageDescription()}Scroll to inspect the full-size filtered image.{/snippet}

<div class="flex flex-col gap-8">
	<OcrBoundingBoxesSetting />
	<OcrGroupingSettings />
	<DictionarySettings />
	<div>
		<h2 class="mb-2 font-medium">Last OCR image</h2>
		{#if ocrDebug.imageUrl}
			<p class="mb-2 text-muted-foreground text-sm">
				{ocrDebug.width} × {ocrDebug.height} pixels
			</p>
			<Dialog
				trigger={imageTrigger}
				title={imageTitle}
				description={imageDescription}
				dialogClose={imageClose}
				contentProps={{ class: 'w-[calc(100vw-2rem)] h-[calc(100vh-2rem)]' }}
			>
				<div class="min-h-0 overflow-auto px-6">
					<img
						src={ocrDebug.imageUrl}
						alt="Latest black and white OCR input at full size"
						class="max-w-none [image-rendering:pixelated]"
					/>
				</div>
			</Dialog>
		{:else}
			<p class="text-muted-foreground text-sm">
				Trigger OCR capture to show its processed image here.
			</p>
		{/if}
	</div>
	{#if isWindows}
		<div>
			<h2 class="mb-2 font-medium">Last filtered relic selection capture</h2>
			{#if relicImageUrl}
				<p class="mb-2 text-muted-foreground text-sm">
					{relicImageSize?.width} × {relicImageSize?.height} pixels ·
					{relicMatchedItem
						? `${relicMatchedItem} (slot ${(relicSelectedSlot ?? 0) + 1})${relicClusterX === null ? '' : ` · cluster x=${relicClusterX}`}`
						: relicClusterX === null
							? `No reward match (${relicRewardCount} mapped of ${relicOcrWordCount} OCR words)`
							: `Cluster x=${relicClusterX} · no reward match (${relicRewardCount} mapped of ${relicOcrWordCount} OCR words)`}
				</p>
				<p class="mb-2 text-muted-foreground text-sm">
					OCR right edges:
					{relicRewardEdges.map(([name, edge], index) => `${index + 1}: ${name} x=${Math.round(edge)}`).join(' · ')}
				</p>
				<p class="mb-2 text-muted-foreground text-sm">{relicStatus}</p>
				<Dialog
					trigger={relicImageTrigger}
					title={relicImageTitle}
					description={relicImageDescription}
					dialogClose={imageClose}
					contentProps={{ class: 'w-[calc(100vw-2rem)] h-[calc(100vh-2rem)]' }}
				>
					<div class="min-h-0 overflow-auto px-6">
						<div class="relative w-max">
							<img
								src={relicImageUrl}
								alt="Last binary filtered relic selection strip at full size"
								class="max-w-none [image-rendering:pixelated]"
							/>
							{#each relicRewardEdges as [, edge]}
								<span class="pointer-events-none absolute top-0 bottom-0 border-l border-amber-500" style:left={`${edge / (relicImageSize?.width ?? 1) * 100}%`}></span>
							{/each}
						</div>
					</div>
				</Dialog>
			{:else}
				<p class="text-muted-foreground text-sm">No relic selection strip has been captured yet.</p>
			{/if}
		</div>
	{/if}
</div>
