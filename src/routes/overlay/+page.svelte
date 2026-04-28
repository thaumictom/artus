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
		ducats?: number;
		trades_24h?: number;
		moving_avg?: number;
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

	const countFormatter = new Intl.NumberFormat(undefined, {
		minimumFractionDigits: 0,
		maximumFractionDigits: 0,
	});

	function normalizeOverlayNumber(value: unknown): number | undefined {
		return typeof value === 'number' && Number.isFinite(value) ? value : undefined;
	}

	function formatOverlayLabel(word: OcrWord): string {
		const lines = [word.text];

		const marketMedian = normalizeOverlayNumber(word.market_median);
		if (marketMedian !== undefined) {
			const suffix = word.market_median_from_current_offers ? '*' : '';
			lines.push(`${medianFormatter.format(marketMedian)}${suffix}`);
		}

		const trades24h = normalizeOverlayNumber(word.trades_24h);
		const movingAvg = normalizeOverlayNumber(word.moving_avg);
		if (movingAvg !== undefined) {
			lines.push(`Avg ${medianFormatter.format(movingAvg)}`);
		}

		if (trades24h !== undefined) {
			lines.push(`24h ${countFormatter.format(trades24h)}`);
		}

		const ducats = normalizeOverlayNumber(word.ducats);
		if (ducats !== undefined) {
			lines.push(`Ducats ${countFormatter.format(ducats)}`);
		}

		return lines.join('\n');
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
