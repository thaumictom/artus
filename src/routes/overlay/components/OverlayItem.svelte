<script lang="ts">
	import Icon from '@iconify/svelte';
	import { fade } from 'svelte/transition';
	import { flyAndScale } from '$lib/transition';
	import { config } from '$lib/settings.svelte';
	import { inventoryNameKey } from '$lib/inventory';
	import type { OcrWord } from './types';

	let { word, selected, showBoundingBoxes, masteredSlugs, ownedBySlug, ownedByName, sessionDeltaBySlug }: {
		word: OcrWord;
		selected: boolean;
		showBoundingBoxes: boolean;
		masteredSlugs: Set<string>;
		ownedBySlug: Map<string, number>;
		ownedByName: Map<string, number>;
		sessionDeltaBySlug: Map<string, number>;
	} = $props();

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

	const ModColor: Record<NonNullable<OcrWord['mod_type']>, string> = {
		gold: 'rgb(253, 235, 189)',
		silver: 'rgb(228, 228, 228)',
		bronze: 'rgb(221, 160, 133)',
		archon: 'rgb(190, 169, 102)',
		special: 'rgb(255, 255, 255)',
	};

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
		if (!tier) return ItemColor.HOLD;

		// if plat is below salvage threshold -> salvage
		// if plat is above sell threshold -> sell
		// else -> hold
		if (plat <= tier.salvage) return ItemColor.SALVAGE;
		if (plat >= tier.sell) return ItemColor.SELL;

		return ItemColor.HOLD;
	}
</script>

{#if word}
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
	{@const primeSetDucats = normalizeOverlayNumber(word.prime_set_ducats)}
	{@const modColor = word.mod_type ? ModColor[word.mod_type] : undefined}

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
	{@const ownedCount =
		(word.slug ? ownedBySlug.get(word.slug) : undefined) ??
		ownedByName.get(inventoryNameKey(word.text)) ??
		0}
	{@const sessionDelta = word.slug ? (sessionDeltaBySlug.get(word.slug) ?? 0) : 0}
	{@const showOwnership = !isCustom || ownedCount > 0 || sessionDelta !== 0}
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
			'selection-ring': selected,
		}}
		style={`left:${word.x + word.width / 2}px;top:${word.y - 16}px;`}
		style:border-color={modColor}
	>
		<div
			class={{
				'border-b text-center font-semibold px-2 py-1 flex flex-col': true,
				'font-stretch-extra-condensed': displayText.length > 30,
				'font-stretch-condensed': displayText.length > 20,
				'font-stretch-semi-condensed': displayText.length > 15,
				'text-muted-foreground': isCustom,
			}}
			style:border-bottom-color={modColor}
		>
			<div>
				<!-- {#if word.vaulted}
					<Icon icon="streamline-flex:safe-vault-solid" class="inline mr-0.5 text-amber-500" />
				{/if} -->
				{#if word.slug && masteredSlugs.has(word.slug)}
					<Icon icon="hugeicons:laurel-wreath-left-03" class="inline size-3.5 text-orange-300" />
				{/if}
				<span class="[text-box-trim:trim-both] [text-box-edge:cap_alphabetic]">
					{displayText}
				</span>
				{#if word.quantity != null}
					<span class="ml-1 text-amber-400">×{word.quantity}</span>
				{/if}
				{#if word.slug && masteredSlugs.has(word.slug)}
					<Icon icon="hugeicons:laurel-wreath-right-03" class="inline size-3.5 text-orange-300" />
				{/if}
			</div>
			{#if word.vaulted || showOwnership}
				<div class="font-medium text-[10px] text-muted-foreground">
					{#if word.vaulted}
						<span class="text-amber-500">vaulted</span>
						{#if showOwnership}<span class="mx-0.5">•</span>{/if}
					{/if}
					{#if showOwnership}
						{ownedCount} owned
						{#if sessionDelta !== 0}
							<span
								class="ml-0.5"
								class:text-accent={sessionDelta > 0}
								class:text-red-400={sessionDelta < 0}
							>
								({sessionDelta > 0 ? '+' : ''}{sessionDelta})
							</span>
						{/if}
					{/if}
				</div>
			{/if}
		</div>
		{#if displayPrice !== undefined || ducats !== undefined || trades24h !== undefined}
			<div class="flex flex-col items-center px-2 py-1 font-medium">
				<div class="flex justify-around gap-1 w-full">
					{#if displayPrice !== undefined}
						<div
							class:col-span-2={trades24h === undefined}
							class="flex justify-center items-center gap-1"
						>
							<div>{pricePrefix}{medianFormatter.format(displayPrice)}</div>
							<img src="/icons/platinum.png" alt="" class="size-3" />
						</div>
					{/if}
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
				{#if trades24h !== undefined}
					<div class="text-xs">
						volume: {countFormatter.format(trades24h)}
					</div>
				{/if}
			</div>
		{/if}
		{#if config.show_max_rank_prices && (!word.is_mod || config.show_max_rank_mod_prices) && maxedArcanePrice !== undefined}
			<div class="flex flex-col items-center px-2 py-1 border-t font-medium">
				<div class="text-[10px] text-muted-foreground">maxed</div>
				<div class="flex justify-center items-center gap-1">
					<div>
						{word.maxed_arcane_price_from_current_offers ? '~' : ''}{medianFormatter.format(
							maxedArcanePrice,
						)}
					</div>
					<img src="/icons/platinum.png" alt="" class="size-3" />
				</div>
				{#if maxedArcaneVolume !== undefined}
					<div class="text-xs">volume: {countFormatter.format(maxedArcaneVolume)}</div>
				{/if}
			</div>
		{/if}
		{#if config.show_set_prices && primeSetPrice !== undefined}
			<div class="flex flex-col items-center px-2 py-1 border-t font-medium">
				<div class="text-[10px] text-muted-foreground">set</div>
				<div class="flex justify-around gap-1 w-full">
					<div class="flex justify-center items-center gap-1">
						<div>
							{word.prime_set_price_from_current_offers ? '~' : ''}{medianFormatter.format(
								primeSetPrice,
							)}
						</div>
						<img src="/icons/platinum.png" alt="" class="size-3" />
					</div>
					{#if primeSetDucats !== undefined}
						<div class="flex justify-center items-center gap-1">
							<div>{countFormatter.format(primeSetDucats)}</div>
							<img src="/icons/ducats.png" alt="" class="size-3" />
						</div>
					{/if}
					{#if primeSetDucats !== undefined && primeSetDucats > 0}
						{@const setPlatPer100Ducats = (primeSetPrice / primeSetDucats) * 100}
						<div>
							<span
								class={getItemActionColor(primeSetDucats, primeSetPrice)}
								title="Platinum per 100 ducats"
							>
								{medianFormatter.format(setPlatPer100Ducats)}
							</span>
						</div>
					{/if}
				</div>
				{#if primeSetVolume !== undefined}
					<div class="text-xs">
						volume: {countFormatter.format(primeSetVolume)}
					</div>
				{/if}
			</div>
		{/if}
	</div>
{/if}

<style>
	.selection-ring::after {
		content: '';
		position: absolute;
		inset: -4px;
		border: 2px solid transparent;
		pointer-events: none;
		background: repeating-linear-gradient(
				90deg,
				var(--color-cyan-400) 0%,
				var(--color-accent) 25%,
				var(--color-cyan-400) 50%
			)
			border-box;
		background-size: 200% 100%;
		mask:
			linear-gradient(#fff 0 0) padding-box,
			linear-gradient(#fff 0 0);
		mask-composite: exclude;
		-webkit-mask:
			linear-gradient(#fff 0 0) padding-box,
			linear-gradient(#fff 0 0);
		-webkit-mask-composite: xor;
		animation: selection-border-flow 2s linear infinite;
	}

	@keyframes selection-border-flow {
		to {
			background-position: 100% 0;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.selection-ring::after {
			animation: none;
		}
	}
</style>
