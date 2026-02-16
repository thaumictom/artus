<script lang="ts">
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';

	interface OcrWordBox {
		text: string;
		left: number;
		top: number;
		right: number;
		bottom: number;
	}

	interface OcrOverlayData {
		window_left: number;
		window_top: number;
		words: OcrWordBox[];
	}

	let overlayWords: OcrWordBox[] = [];
	let windowLeft = 0;
	let windowTop = 0;
	let showOverlay = false;
	let timeoutId: number;

	onMount(() => {
		const unlistenOverlay = listen<OcrOverlayData>('ocr-overlay', ({ payload }) => {
			windowLeft = payload.window_left;
			windowTop = payload.window_top;
			overlayWords = payload.words;
			showOverlay = true;

			clearTimeout(timeoutId);
			timeoutId = window.setTimeout(() => {
				showOverlay = false;
			}, 15000);
		});

		return () => {
			unlistenOverlay.then((f) => f());
		};
	});
</script>

<main
	class="fixed inset-0 pointer-events-none transition-opacity duration-200 {showOverlay
		? 'opacity-100'
		: 'opacity-0'}"
>
	{#if showOverlay}
		{#each overlayWords as word}
			<div
				class="absolute bg-black/70 px-1 border border-green-400 rounded font-mono text-green-400 text-sm whitespace-nowrap"
				style="left: {windowLeft + word.left}px; top: {windowTop + word.top}px;"
			>
				{word.text}
			</div>
		{/each}
	{/if}
</main>
