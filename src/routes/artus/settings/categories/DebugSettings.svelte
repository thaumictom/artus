<script lang="ts">
	import { ocrDebug } from '$lib/ocr-debug.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import DictionarySettings from '../components/DictionarySettings.svelte';
	import OcrBoundingBoxesSetting from '../components/OcrBoundingBoxesSetting.svelte';
	import OcrGroupingSettings from '../components/OcrGroupingSettings.svelte';
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
</div>
