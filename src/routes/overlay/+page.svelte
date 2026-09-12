<script lang="ts">
	import Icon from '@iconify/svelte';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import { flyAndScale } from '$lib/transition';
	import { config, loadSettings } from '$lib/settings.svelte';

	type OcrWord = {
		text: string;
		x: number;
		y: number;
		width: number;
		height: number;
		market_median?: number;
		market_median_from_current_offers?: boolean;
		maxed_arcane_price?: number;
		maxed_arcane_trades_24h?: number;
		maxed_arcane_price_from_current_offers?: boolean;
		prime_set_price?: number;
		prime_set_trades_24h?: number;
		prime_set_price_from_current_offers?: boolean;
		relic_price_is_fallback?: boolean;
		ducats?: number;
		vaulted?: boolean;
		is_custom?: boolean;
		is_relic?: boolean;
		subtype?: string;
		trades_24h?: number;
		moving_avg?: number;
		mod_type?: 'gold' | 'silver' | 'bronze' | 'archon' | 'special';
	};

	let words: OcrWord[] = $state([]);
	let showBoundingBoxes = $state(false);
	let processing = $state(false);

	onMount(() => {
		loadSettings();
		const cleanups: Array<() => void> = [];

		listen('ocr_processing', () => {
			words = [];
			processing = true;
		}).then((cleanup) => cleanups.push(cleanup));

		listen<{ words: OcrWord[]; show_ocr_bounding_boxes: boolean }>('ocr_result', (event) => {
			processing = false;
			words = event.payload?.words ?? [];
			showBoundingBoxes = event.payload?.show_ocr_bounding_boxes ?? false;
			// Reload settings to get the latest thresholds if changed
			loadSettings();
		}).then((cleanup) => cleanups.push(cleanup));

		listen('ocr_clear', () => {
			words = [];
			processing = false;
		}).then((cleanup) => cleanups.push(cleanup));

		return () => {
			for (const cleanup of cleanups) cleanup();
		};
	});

	const medianFormatter = new Intl.NumberFormat(undefined, {
		minimumFractionDigits: 0,
		maximumFractionDigits: 1,
	});

	const countFormatter = new Intl.NumberFormat(undefined, {
		minimumFractionDigits: 0,
		maximumFractionDigits: 0,
	});

	function normalizeOverlayNumber(value: unknown): number | undefined {
		return typeof value === 'number' && Number.isFinite(value) ? value : undefined;
	}

	const ItemColor = {
		SALVAGE: 'text-amber-500',
		SELL: 'text-cyan-500',
		HOLD: 'text-muted-foreground',
	} as const;

	function getItemActionColor(
		ducats: number,
		plat: number,
		minTradeValue = 5,
	): (typeof ItemColor)[keyof typeof ItemColor] {
		if (plat < minTradeValue) return ItemColor.SALVAGE;

		const thresholds = {
			100: { salvage: config.threshold_100[0], sell: config.threshold_100[1] },
			65: { salvage: config.threshold_65[0], sell: config.threshold_65[1] },
			45: { salvage: config.threshold_45[0], sell: config.threshold_45[1] },
			25: { salvage: config.threshold_25[0], sell: config.threshold_25[1] },
			15: { salvage: config.threshold_15[0], sell: config.threshold_15[1] },
		};

		const tier = thresholds[ducats as keyof typeof thresholds];

		// if plat is below salvage threshold -> salvage
		// if plat is above sell threshold -> sell
		// else -> hold
		if (plat <= tier.salvage) return ItemColor.SALVAGE;
		if (plat >= tier.sell) return ItemColor.SELL;

		return ItemColor.HOLD;
	}
</script>

<main class="relative w-screen h-screen pointer-events-none">
	{#if processing}
		<div
			in:flyAndScale={{ y: 24 }}
			out:fade={{ duration: 100 }}
			class="absolute inset-0 flex justify-center items-center"
		>
			<div class="flex items-center gap-4 bg-background/90 p-4 border">
				<Icon icon="material-symbols:progress-activity" class="size-5 animate-spin" />
				<span class="text-foreground text-sm">Processing…</span>
			</div>
		</div>
	{/if}
	{#each words as word (`${word.text}-${word.x}-${word.y}-${word.width}-${word.height}`)}
		{@const marketMedian = normalizeOverlayNumber(word.market_median)}
		{@const movingAvg = normalizeOverlayNumber(word.moving_avg)}
		{@const displayPrice = word.market_median_from_current_offers
			? marketMedian
			: (movingAvg ?? marketMedian)}
		{@const pricePrefix = word.market_median_from_current_offers ? '~' : ''}
		{@const trades24h = normalizeOverlayNumber(word.trades_24h)}
		{@const ducats = normalizeOverlayNumber(word.ducats)}
		{@const maxedArcanePrice = normalizeOverlayNumber(word.maxed_arcane_price)}
		{@const maxedArcaneVolume = normalizeOverlayNumber(word.maxed_arcane_trades_24h)}
		{@const primeSetPrice = normalizeOverlayNumber(word.prime_set_price)}
		{@const primeSetVolume = normalizeOverlayNumber(word.prime_set_trades_24h)}
		{@const relatedVolume = maxedArcanePrice !== undefined ? maxedArcaneVolume : primeSetVolume}

		<!-- Determine the actual displayed name of the relic based on which price we fell back to. -->
		{@const isOriginallyRadiant = word.subtype === 'Radiant'}
		{@const baseText = word.is_relic
			? word.text.replace(/ \[Exceptional\]| \[Flawless\]| \[Radiant\]/, '')
			: word.text}
		{@const showRadiant = word.relic_price_is_fallback ? !isOriginallyRadiant : isOriginallyRadiant}
		{@const displayText = word.is_relic
			? showRadiant
				? `${baseText} [Radiant]`
				: baseText
			: word.text}

		{@const isCustom = word.is_custom === true}
		<!-- Bounding box for debugging -->
		{#if showBoundingBoxes}
			<div
				in:fade={{ duration: 200 }}
				class="absolute border border-red-500 text-red-500/25 striped-gradient"
				style={`left:${word.x}px;top:${word.y}px;width:${word.width}px;height:${word.height}px;`}
			></div>
		{/if}
		<div
			in:flyAndScale={{ y: 24 }}
			out:fade={{ duration: 100 }}
			class={{
				'absolute flex flex-col bg-background/90 border text-foreground text-sm -translate-x-1/2 -translate-y-full': true,
				'border-[rgb(253,235,189)] text-[rgb(253,235,189)]': word.mod_type === 'gold',
				'border-[rgb(228,228,228)] text-[rgb(228,228,228)]': word.mod_type === 'silver',
				'border-[rgb(221,160,133)] text-[rgb(221,160,133)]': word.mod_type === 'bronze',
				'border-[rgb(190,169,102)] text-[rgb(190,169,102)]': word.mod_type === 'archon',
				'border-[rgb(255,255,255)] text-[rgb(255,255,255)]': word.mod_type === 'special',
			}}
			style={`left:${word.x + word.width / 2}px;top:${word.y - 16}px;`}
		>
			<div
				class={{
					'border-b text-center font-semibold': true,
					'font-stretch-extra-condensed': displayText.length > 30,
					'font-stretch-condensed': displayText.length > 20,
					'font-stretch-semi-condensed': displayText.length > 15,
					'text-muted-foreground': isCustom,
				}}
			>
				{#if word.vaulted}
					<Icon icon="streamline-flex:safe-vault-solid" class="inline mr-0.5 text-amber-500" />
				{/if}
				<span>{displayText}</span>
			</div>
			<div class="items-center gap-x-2 grid grid-cols-2 font-medium text-center">
				<!-- Line 1 -->
				{#if displayPrice !== undefined}
					<div
						class:col-span-2={trades24h === undefined}
						class="flex justify-center items-center gap-1"
					>
						<div>{pricePrefix}{medianFormatter.format(displayPrice)}</div>
						<img src="/icons/platinum.png" alt="" class="size-3" />
					</div>
				{/if}
				{#if trades24h !== undefined}
					<div class:col-span-2={displayPrice === undefined} class="text-xs">
						vol: {countFormatter.format(trades24h)}
					</div>
				{/if}
				{#if maxedArcanePrice !== undefined || primeSetPrice !== undefined}
					<!-- Line 2 -->
					<div class="col-span-2 text-[10px] text-muted-foreground text-center">
						{maxedArcanePrice !== undefined ? 'max' : 'set'}
					</div>
					<!-- Line 3 -->
					<div
						class:col-span-2={relatedVolume === undefined}
						class="flex justify-center items-center gap-1"
					>
						{#if maxedArcanePrice !== undefined}
							{word.maxed_arcane_price_from_current_offers ? '~' : ''}{medianFormatter.format(
								maxedArcanePrice,
							)}
						{:else if primeSetPrice !== undefined}
							{word.prime_set_price_from_current_offers ? '~' : ''}{medianFormatter.format(
								primeSetPrice,
							)}
						{/if}
						<img src="/icons/platinum.png" alt="" class="size-3" />
					</div>
					{#if relatedVolume !== undefined}
						<div class="text-xs">vol: {countFormatter.format(relatedVolume)}</div>
					{/if}
				{/if}
				<!-- Line 4 -->
				{#if ducats !== undefined}
					<div class="flex justify-center items-center gap-1">
						<div>{countFormatter.format(ducats)}</div>
						<img src="/icons/ducats.png" alt="" class="size-3" />
					</div>
				{/if}
				{#if displayPrice !== undefined && ducats !== undefined && ducats > 0}
					{@const platPer100Ducats = (displayPrice / ducats) * 100}
					<div>
						<span class={getItemActionColor(ducats, displayPrice)}>
							{medianFormatter.format(platPer100Ducats)}
						</span>
					</div>
				{/if}
			</div>
		</div>
	{/each}
</main>
