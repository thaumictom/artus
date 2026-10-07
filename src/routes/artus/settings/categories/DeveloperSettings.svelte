<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { platform } from '@tauri-apps/plugin-os';
	import { onMount } from 'svelte';
	import { ocrDeveloper } from '$lib/ocr-developer.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import Switch from '$lib/components/Switch.svelte';
	import Slider from '$lib/components/Slider.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
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
	let savingDeveloperConsole = $state(false);
	let developerConsoleError = $state(false);
	async function setDeveloperConsole(enabled: boolean) {
		savingDeveloperConsole = true;
		try {
			await invoke('set_developer_console_enabled', { enabled });
			config.developer_console_enabled = enabled;
			developerConsoleError = false;
		} catch (error) {
			console.error('Could not update developer console:', error);
			developerConsoleError = true;
		} finally {
			savingDeveloperConsole = false;
		}
	}
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
				const image = await invoke<RelicSelectionImage | null>(
					'get_relic_selection_developer_image',
				);
				if (disposed || request !== requestId || !image) return;
				const nextUrl = URL.createObjectURL(
					new Blob([new Uint8Array(image.png_bytes)], { type: 'image/png' }),
				);
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
			src={ocrDeveloper.imageUrl ?? ''}
			alt="Latest black and white OCR input"
			class="border border-border rounded w-full [image-rendering:pixelated]"
		/>
	</button>
{/snippet}

{#snippet imageTitle()}OCR image{/snippet}
{#snippet imageDescription()}Scroll to inspect the full-size image.{/snippet}
{#snippet imageClose()}<Button>Close</Button>{/snippet}

{#snippet relicImageTrigger()}
	<button class="block w-full cursor-zoom-in" aria-label="Enlarge relic selection capture">
		<span class="block relative">
			<img
				src={relicImageUrl ?? ''}
				alt="Last binary filtered relic selection strip"
				class="border border-border w-full [image-rendering:pixelated]"
			/>
			{#each relicRewardEdges as [, edge]}
				<span
					class="top-0 bottom-0 absolute border-amber-500 border-l pointer-events-none"
					style:left={`${(edge / (relicImageSize?.width ?? 1)) * 100}%`}
				></span>
			{/each}
		</span>
	</button>
{/snippet}

{#snippet relicImageTitle()}Relic selection capture{/snippet}
{#snippet relicImageDescription()}Scroll to inspect the full-size filtered image.{/snippet}

<div class="flex flex-col">
	<CommonSetting
		title="Enable developer console"
		description="Allow webview developer tools. Use Ctrl+Shift+I to open them."
		labelProps={{ for: 'developer-console-enabled' }}
	>
		<Switch
			id="developer-console-enabled"
			checked={config.developer_console_enabled}
			disabled={savingDeveloperConsole}
			onCheckedChange={setDeveloperConsole}
		/>
	</CommonSetting>
	{#if developerConsoleError}<p role="alert" class="p-4 text-danger text-sm">
			Could not update the developer console. Try again.
		</p>{/if}
	<CommonSetting
		title="Show unused dashboard views"
		description="Show the Unused group and its pinned views in dashboard navigation. These views may have no useful information yet."
		labelProps={{ for: 'show_unused_dashboard_views' }}
	>
		<Switch
			id="show_unused_dashboard_views"
			bind:checked={config.show_unused_dashboard_views}
			onCheckedChange={() => updateSetting('show_unused_dashboard_views')}
		/>
	</CommonSetting>
	<OcrBoundingBoxesSetting />
	<OcrGroupingSettings />
	<DictionarySettings />
	<CommonSetting
		title="Quantity checkmark confidence"
		description="Minimum checkmark match confidence for reading owned quantities. Lower it for missed checkmarks; raise it if other shapes are detected. Changes apply to the next OCR capture."
		align="vertical"
	>
		<Slider
			min={0.5}
			max={1}
			step={0.01}
			type="single"
			aria-label="Quantity checkmark confidence"
			onValueCommit={() => updateSetting('ocr_checkmark_match_threshold')}
			bind:value={config.ocr_checkmark_match_threshold}
		>
			{#snippet thumbLabel({ value })}
				{Math.round(
					(typeof value === 'number' ? value : config.ocr_checkmark_match_threshold) * 100,
				)}%
			{/snippet}
		</Slider>
		<p class="text-muted-foreground text-base">
			Current confidence: {Math.round(config.ocr_checkmark_match_threshold * 100)}%
		</p>
	</CommonSetting>
	<CommonSetting
		title="Hide donate button"
		description="Hide the Donate button in the app header."
		labelProps={{ for: 'hide_donate_button' }}
	>
		<Switch
			id="hide_donate_button"
			onCheckedChange={() => updateSetting('hide_donate_button')}
			bind:checked={config.hide_donate_button}
		/>
	</CommonSetting>
	<div class="p-4 border border-border-secondary">
		<h2 class="mb-2 font-medium">Last OCR image</h2>
		{#if ocrDeveloper.imageUrl}
			<p class="mb-2 text-muted-foreground text-base">
				{ocrDeveloper.width} × {ocrDeveloper.height} pixels
			</p>
			<Dialog
				trigger={imageTrigger}
				title={imageTitle}
				description={imageDescription}
				dialogClose={imageClose}
				contentProps={{ class: 'w-[calc(100vw-2rem)] h-[calc(100vh-2rem)]' }}
			>
				<div class="px-6 min-h-0 overflow-auto">
					<img
						src={ocrDeveloper.imageUrl}
						alt="Latest black and white OCR input at full size"
						class="max-w-none [image-rendering:pixelated]"
					/>
				</div>
			</Dialog>
		{:else}
			<p class="text-muted-foreground text-base">
				Trigger OCR capture to show its processed image here.
			</p>
		{/if}
	</div>
	{#if isWindows}
		<div class="-mt-px p-4 border border-border-secondary">
			<h2 class="mb-2 font-medium">Last filtered relic selection capture</h2>
			{#if relicImageUrl}
				<p class="mb-2 text-muted-foreground text-base">
					{relicImageSize?.width} × {relicImageSize?.height} pixels ·
					{relicMatchedItem
						? `${relicMatchedItem} (slot ${(relicSelectedSlot ?? 0) + 1})${relicClusterX === null ? '' : ` · cluster x=${relicClusterX}`}`
						: relicClusterX === null
							? `No reward match (${relicRewardCount} mapped of ${relicOcrWordCount} OCR words)`
							: `Cluster x=${relicClusterX} · no reward match (${relicRewardCount} mapped of ${relicOcrWordCount} OCR words)`}
				</p>
				<p class="mb-2 text-muted-foreground text-base">
					OCR right edges:
					{relicRewardEdges
						.map(([name, edge], index) => `${index + 1}: ${name} x=${Math.round(edge)}`)
						.join(' · ')}
				</p>
				<p class="mb-2 text-muted-foreground text-base">{relicStatus}</p>
				<Dialog
					trigger={relicImageTrigger}
					title={relicImageTitle}
					description={relicImageDescription}
					dialogClose={imageClose}
					contentProps={{ class: 'w-[calc(100vw-2rem)] h-[calc(100vh-2rem)]' }}
				>
					<div class="px-6 min-h-0 overflow-auto">
						<div class="relative w-max">
							<img
								src={relicImageUrl}
								alt="Last binary filtered relic selection strip at full size"
								class="max-w-none [image-rendering:pixelated]"
							/>
							{#each relicRewardEdges as [, edge]}
								<span
									class="top-0 bottom-0 absolute border-amber-500 border-l pointer-events-none"
									style:left={`${(edge / (relicImageSize?.width ?? 1)) * 100}%`}
								></span>
							{/each}
						</div>
					</div>
				</Dialog>
			{:else}
				<p class="text-muted-foreground text-base">
					No relic selection strip has been captured yet.
				</p>
			{/if}
		</div>
	{/if}
</div>
