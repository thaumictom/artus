<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';

	type OcrWord = {
		text: string;
		x: number;
		y: number;
		width: number;
		height: number;
		market_median?: number;
		market_median_from_current_offers?: boolean;
	};

	let words: OcrWord[] = $state([]);

	onMount(() => {
		let unlisten: (() => void) | undefined;

		listen<{ words: OcrWord[] }>('ocr_result', (event) => {
			words = event.payload?.words ?? [];
		}).then((cleanup) => {
			unlisten = cleanup;
		});

		return () => {
			unlisten?.();
		};
	});

	const medianFormatter = new Intl.NumberFormat(undefined, {
		minimumFractionDigits: 0,
		maximumFractionDigits: 2,
	});

	function formatOverlayLabel(word: OcrWord): string {
		if (typeof word.market_median !== 'number' || !Number.isFinite(word.market_median)) {
			return word.text;
		}

		const suffix = word.market_median_from_current_offers ? '*' : '';
		return `${word.text}\n${medianFormatter.format(word.market_median)}${suffix}`;
	}
</script>

<main class="relative w-screen h-screen pointer-events-none">
	{#each words as word (`${word.text}-${word.x}-${word.y}-${word.width}-${word.height}`)}
		<div
			in:fade={{ duration: 200 }}
			class="absolute bg-black/70 p-1 border rounded-sm font-semibold text-teal-400 text-sm whitespace-pre-line"
			style={`left:${word.x}px;top:${word.y}px;`}
		>
			{formatOverlayLabel(word)}
		</div>
	{/each}
</main>
