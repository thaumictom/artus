<script lang="ts">
	import Icon from '@iconify/svelte';
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
		vaulted?: boolean;
		is_custom?: boolean;
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
</script>

<main class="relative w-screen h-screen pointer-events-none">
	{#each words as word (`${word.text}-${word.x}-${word.y}-${word.width}-${word.height}`)}
		{@const marketMedian = normalizeOverlayNumber(word.market_median)}
		{@const inaccurateMarker = word.market_median_from_current_offers ? '~' : ''}
		{@const movingAvg = normalizeOverlayNumber(word.moving_avg)}
		{@const trades24h = normalizeOverlayNumber(word.trades_24h)}
		{@const ducats = normalizeOverlayNumber(word.ducats)}

		{@const vaulted = word.vaulted}
		{@const isCustom = word.is_custom === true}
		<div
			in:fade={{ duration: 200 }}
			class="absolute flex flex-col bg-background/75 px-2 py-1 border text-foreground text-sm -translate-x-1/2"
			style={`left:${word.x + word.width / 2}px;top:${word.y + word.height + 16}px;`}
		>
			<div
				class={{
					'font-semibold text-center mb-0.5': true,
					'font-stretch-condensed': word.text.length > 30,
					'font-stretch-semi-condensed': word.text.length > 20,
					'text-muted-foreground': isCustom || vaulted,
				}}
			>
				{word.text}
			</div>
			{#if movingAvg !== undefined || ducats !== undefined}
				<div class="flex justify-around gap-1">
					{#if movingAvg !== undefined}
						<div class="flex items-center gap-1">
							<div>{inaccurateMarker}{medianFormatter.format(movingAvg)}</div>
							<img src="/icons/platinum.png" alt="" class="size-3" />
						</div>
					{/if}
					{#if ducats !== undefined}
						<div class="flex items-center gap-1">
							<div>{countFormatter.format(ducats)}</div>
							<img src="/icons/ducats.png" alt="" class="size-3" />
						</div>
					{/if}
				</div>
			{/if}
			{#if trades24h !== undefined}
				<div class="text-xs text-center">
					volume: {countFormatter.format(trades24h)}
				</div>
			{/if}
		</div>
	{/each}
</main>
