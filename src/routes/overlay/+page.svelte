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
		<div
			in:fade={{ duration: 200 }}
			class="absolute flex flex-col bg-black/75 px-2 py-1 border text-foreground text-sm -translate-x-1/2"
			style={`left:${word.x + word.width / 2}px;top:${word.y + word.height + 16}px;`}
		>
			<div
				class={{
					'font-bold text-center': true,
					'text-muted-foreground': vaulted,
				}}
			>
				{word.text}
			</div>
			{#if movingAvg !== undefined || ducats !== undefined}
				<div class="flex justify-around gap-1">
					{#if movingAvg !== undefined}
						<div class="flex gap-1">
							{inaccurateMarker}{medianFormatter.format(movingAvg)}p
						</div>
					{/if}
					{#if ducats !== undefined}
						<div class="flex gap-1">
							<Icon icon="simple-icons:ducati" class="size-5"></Icon>
							{countFormatter.format(ducats)}
						</div>
					{/if}
				</div>
			{/if}
			{#if trades24h !== undefined}
				<div class="text-xs text-center">
					{countFormatter.format(trades24h)} in 24h
				</div>
			{/if}
		</div>
	{/each}
</main>
