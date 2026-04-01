<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	type OcrDebugImagePayload = {
		png_bytes: number[];
		width: number;
		height: number;
		upscale_amount: number;
	};

	type OcrTextPayload = {
		text: string;
	};

	let hotkey = 'Home';
	let status = '';
	let debugImageUrl = '';
	let debugImageInfo = '';
	let ocrText = '';
	let isImageFullscreen = false;

	onMount(() => {
		let unlistenImage: (() => void) | undefined;
		let unlistenText: (() => void) | undefined;
		let activeUrl = '';

		listen<OcrDebugImagePayload>('ocr_debug_image', (event) => {
			const payload = event.payload;
			const bytes = new Uint8Array(payload.png_bytes);
			const blob = new Blob([bytes], { type: 'image/png' });
			const nextUrl = URL.createObjectURL(blob);
			if (activeUrl) URL.revokeObjectURL(activeUrl);
			activeUrl = nextUrl;
			debugImageUrl = nextUrl;
			debugImageInfo = `${payload.width}x${payload.height} (upscale ${payload.upscale_amount}x)`;
		}).then((cleanup) => {
			unlistenImage = cleanup;
		});

		listen<OcrTextPayload>('ocr_text_result', (event) => {
			ocrText = event.payload?.text ?? '';
		}).then((cleanup) => {
			unlistenText = cleanup;
		});

		(async () => {
			try {
				hotkey = await invoke<string>('get_hotkey');
			} catch (error) {
				status = String(error);
			}
		})();

		return () => {
			unlistenImage?.();
			unlistenText?.();
			if (activeUrl) URL.revokeObjectURL(activeUrl);
		};
	});

	async function saveHotkey() {
		status = '';
		try {
			await invoke('set_hotkey', { hotkey });
			status = 'Saved';
		} catch (error) {
			status = String(error);
		}
	}

	function openImageFullscreen() {
		isImageFullscreen = true;
	}

	function closeImageFullscreen() {
		isImageFullscreen = false;
	}
</script>

<section class="max-w-md">
	<p class="text-sm">Set global shortcut (examples: Home, Ctrl+Shift+H).</p>

	<div class="flex items-center gap-2 mt-4">
		<input class="px-2 py-1 border rounded w-full" bind:value={hotkey} placeholder="Home" />
		<button class="px-3 py-1 border rounded" onclick={saveHotkey}>Save</button>
	</div>

	{#if status}
		<p class="mt-2 text-sm">{status}</p>
	{/if}

	{#if debugImageUrl}
		<div class="mt-4">
			<p class="text-sm">OCR Input ({debugImageInfo})</p>
			<button
				type="button"
				class="bg-transparent mt-2 p-0 border rounded w-full cursor-zoom-in"
				onclick={openImageFullscreen}
			>
				<img src={debugImageUrl} alt="OCR debug input" class="w-full" />
			</button>
		</div>
	{/if}

	{#if ocrText}
		<div class="mt-4">
			<p class="text-sm">OCR Text</p>
			<textarea
				class="mt-2 p-2 border rounded w-full h-40 text-sm"
				readonly
				value={ocrText}
			></textarea>
		</div>
	{/if}
</section>

{#if isImageFullscreen && debugImageUrl}
	<div
		class="z-50 fixed inset-0 flex justify-center items-center bg-black/90 p-4"
		role="button"
		tabindex="0"
		onclick={closeImageFullscreen}
		onkeydown={(e) =>
			e.key === 'Enter' || e.key === ' ' || e.key === 'Escape' ? closeImageFullscreen() : null}
	>
		<button
			class="top-4 right-4 absolute bg-white px-3 py-1 border rounded text-black"
			onclick={closeImageFullscreen}
		>
			Close
		</button>
		<div
			role="button"
			tabindex="0"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
		>
			<img
				src={debugImageUrl}
				alt="OCR debug input fullscreen"
				class="max-w-full max-h-full object-none"
			/>
		</div>
	</div>
{/if}
