<script lang="ts">
	import Icon from '@iconify/svelte';
	import { listen } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/core';
	import { LazyStore } from '@tauri-apps/plugin-store';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import { flyAndScale } from '$lib/transition';
	import { config, loadSettings, watchOverlayPriceSettings } from '$lib/settings.svelte';
	import { changeOcrItemQuantities, inventoryNameKey, type InventoryItem } from '$lib/inventory';

	type OcrWord = {
		text: string;
		quantity?: number;
		slug?: string;
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
		prime_set_ducats?: number;
		relic_price_is_fallback?: boolean;
		ducats?: number;
		vaulted?: boolean;
		is_custom?: boolean;
		is_relic?: boolean;
		is_mod?: boolean;
		subtype?: string;
		trades_24h?: number;
		moving_avg?: number;
		mod_type?: 'gold' | 'silver' | 'bronze' | 'archon' | 'special';
	};

	let words: OcrWord[] = $state([]);
	let showBoundingBoxes = $state(false);
	let processing = $state(false);
	let controlsEnabled = $state(false);
	let selectedIndex = $state<number | null>(null);
	let sessionDeltaBySlug = $state(new Map<string, number>());
	let overlaySession = 0;
	let relicFeedback = $state<string | null>(null);
	let relicFeedbackGeneration = 0;
	let masteredSlugs = $state(new Set<string>());
	let ownedBySlug = $state(new Map<string, number>());
	let ownedByName = $state(new Map<string, number>());
	const masteryStore = new LazyStore('mastery.json');
	const inventoryStore = new LazyStore('inventory.json');
	let masteryReadSequence = 0;
	let inventoryReadSequence = 0;
	const visualRows = $derived(groupVisualRows(words));
	const cycleOrder = $derived(visualRows.flat());
	type HeldHotkey = {
		delay: ReturnType<typeof setTimeout>;
		interval?: ReturnType<typeof setInterval>;
	};
	const heldHotkeys = new Map<string, HeldHotkey>();
	const repeatableActions = new Set([
		'cycle', 'cycle_back', 'navigate_up', 'navigate_left', 'navigate_down', 'navigate_right',
	]);

	function stopHotkey(action: string) {
		const held = heldHotkeys.get(action);
		if (!held) return;
		clearTimeout(held.delay);
		if (held.interval) clearInterval(held.interval);
		heldHotkeys.delete(action);
	}

	function stopAllHotkeys() {
		for (const action of heldHotkeys.keys()) stopHotkey(action);
	}

	function onHotkeyEvent(action: string, pressed: boolean) {
		if (!controlsEnabled) return;
		if (!pressed) {
			stopHotkey(action);
			return;
		}
		if (heldHotkeys.has(action)) return;
		handleOverlayHotkey(action);
		const held: HeldHotkey = {
			delay: setTimeout(() => {
				if (heldHotkeys.get(action) !== held) return;
				held.interval = setInterval(() => handleOverlayHotkey(action), 85);
			}, 320),
		};
		heldHotkeys.set(action, held);
		if (!repeatableActions.has(action)) clearTimeout(held.delay);
	}

	function groupVisualRows(items: OcrWord[]): number[][] {
		if (items.length === 0) return [];
		const heights = items.map((item) => item.height).sort((a, b) => a - b);
		const rowTolerance = Math.max(8, heights[Math.floor(heights.length / 2)] * 0.75);
		const positioned = items.map((item, index) => ({
			index,
			x: item.x + item.width / 2,
			y: item.y + item.height / 2,
		})).sort((a, b) => a.y - b.y || a.x - b.x);
		const rows: { center: number; members: typeof positioned }[] = [];
		for (const item of positioned) {
			const row = rows.find((candidate) => Math.abs(candidate.center - item.y) <= rowTolerance);
			if (row) {
				row.center = (row.center * row.members.length + item.y) / (row.members.length + 1);
				row.members.push(item);
			} else {
				rows.push({ center: item.y, members: [item] });
			}
		}
		return rows.map((row) => row.members.sort((a, b) => a.x - b.x).map((item) => item.index));
	}

	async function applyInventoryChanges(changes: { word: OcrWord; delta: number }[]) {
		const session = overlaySession;
		try {
			const applied = await changeOcrItemQuantities(changes);
			if (session !== overlaySession) return;
			const updated = new Map(sessionDeltaBySlug);
			for (const [slug, delta] of applied) {
				updated.set(slug, (updated.get(slug) ?? 0) + delta);
			}
			sessionDeltaBySlug = updated;
		} catch (error) {
			console.error('Could not update inventory from overlay:', error);
		}
	}

	function handleOverlayHotkey(action: string) {
		if (!controlsEnabled || processing || words.length === 0) return;
		if (action === 'cycle' || action === 'cycle_back') {
			const position = selectedIndex === null ? -1 : cycleOrder.indexOf(selectedIndex);
			selectedIndex = action === 'cycle_back'
				? cycleOrder[position < 0 ? cycleOrder.length - 1 : (position - 1 + cycleOrder.length) % cycleOrder.length]
				: cycleOrder[(position + 1) % cycleOrder.length];
			return;
		}
		if (action.startsWith('navigate_')) {
			if (selectedIndex === null) { selectedIndex = cycleOrder[0]; return; }
			const current = words[selectedIndex];
			const x = current.x + current.width / 2;
			const y = current.y + current.height / 2;
			const direction = action.slice('navigate_'.length);
			if (direction === 'up' || direction === 'down') {
				const rowIndex = visualRows.findIndex((row) => row.includes(selectedIndex!));
				const nextRow = visualRows[rowIndex + (direction === 'up' ? -1 : 1)];
				if (!nextRow) return;
				// Stay near the same horizontal position when moving to an adjacent row.
				selectedIndex = nextRow.reduce((closest, index) => {
					const distance = Math.abs(words[index].x + words[index].width / 2 - x);
					const closestDistance = Math.abs(words[closest].x + words[closest].width / 2 - x);
					return distance < closestDistance ? index : closest;
				});
				return;
			}
			const candidates = words.map((word, index) => ({
				index,
				dx: word.x + word.width / 2 - x,
				dy: word.y + word.height / 2 - y,
			})).filter((item) => item.index !== selectedIndex);
			const ahead = candidates.filter((item) => direction === 'left' ? item.dx < -1 : item.dx > 1);
			if (ahead.length === 0) return;
			ahead.sort((a, b) => {
				const score = (item: typeof a) => {
					return Math.abs(item.dx) + Math.abs(item.dy) * 2;
				};
				return score(a) - score(b);
			});
			selectedIndex = ahead[0].index;
			return;
		}
		if (action === 'inventory_add_all') {
			void applyInventoryChanges(words.map((word) => ({
				word,
				delta: word.quantity != null && Number.isSafeInteger(word.quantity) && word.quantity > 0
					? word.quantity : 1,
			})));
			return;
		}
		if (selectedIndex === null) return;
		if (action === 'inventory_increment' || action === 'inventory_decrement') {
			void applyInventoryChanges([{
				word: words[selectedIndex],
				delta: action === 'inventory_increment' ? 1 : -1,
			}]);
		}
	}

	function shortcut(action: keyof typeof config.hotkeys) {
		return (config.hotkeys[action] ?? '').toUpperCase();
	}

	async function refreshMasteredSlugs(sequence: number) {
		try {
			const slugs = await masteryStore.get<string[]>('masteredSlugs');
			if (sequence === masteryReadSequence) masteredSlugs = new Set(slugs ?? []);
		} catch (error) {
			console.error('Could not read overlay mastery progress:', error);
		}
	}

	function updateOwnedCounts(items: InventoryItem[]) {
		const slugs = new Map<string, number>();
		const names = new Map<string, number>();
		for (const item of items) {
			if (!Number.isFinite(item.quantity) || item.quantity <= 0) continue;
			if (item.slug) slugs.set(item.slug, (slugs.get(item.slug) ?? 0) + item.quantity);
			const name = inventoryNameKey(item.name);
			names.set(name, (names.get(name) ?? 0) + item.quantity);
		}
		ownedBySlug = slugs;
		ownedByName = names;
	}

	async function refreshInventory(sequence: number) {
		try {
			const items = await inventoryStore.get<InventoryItem[]>('items');
			if (sequence === inventoryReadSequence) updateOwnedCounts(items ?? []);
		} catch (error) {
			console.error('Could not read overlay inventory:', error);
		}
	}

	onMount(() => {
		loadSettings();
		const relicDetectionSound = new Audio('/sound/beep.wav');
		relicDetectionSound.preload = 'auto';
		const cleanups: Array<() => void> = [];
		let disposed = false;
		let relicFeedbackTimer: ReturnType<typeof setTimeout> | undefined;
		const registerCleanup = (cleanup: () => void) => {
			if (disposed) cleanup();
			else cleanups.push(cleanup);
		};
		watchOverlayPriceSettings().then(registerCleanup);
		void refreshMasteredSlugs(masteryReadSequence);
		void refreshInventory(inventoryReadSequence);
		inventoryStore
			.onChange<InventoryItem[]>((key, value) => {
				if (key === 'items') {
					inventoryReadSequence++;
					updateOwnedCounts(Array.isArray(value) ? value : []);
				}
			})
			.then(registerCleanup);

		listen('ocr_processing', () => {
			relicFeedbackGeneration++;
			clearTimeout(relicFeedbackTimer);
			relicFeedback = null;
			stopAllHotkeys();
			overlaySession++;
			sessionDeltaBySlug = new Map();
			masteryReadSequence++;
			words = [];
			processing = true;
			controlsEnabled = false;
			selectedIndex = null;
		}).then(registerCleanup);

		listen<{ words: OcrWord[]; show_ocr_bounding_boxes: boolean; controls_enabled: boolean }>(
			'ocr_result',
			(event) => {
				stopAllHotkeys();
				overlaySession++;
				sessionDeltaBySlug = new Map();
				processing = false;
				controlsEnabled = event.payload?.controls_enabled ?? false;
				words = event.payload?.words ?? [];
				selectedIndex = null;
				// One small store lookup replaces a full catalog load and relationship scan.
				void refreshMasteredSlugs(++masteryReadSequence);
				void refreshInventory(++inventoryReadSequence);
				showBoundingBoxes = event.payload?.show_ocr_bounding_boxes ?? false;
				// Reload settings to get the latest thresholds if changed
				loadSettings();
			},
		).then(registerCleanup);

		listen('ocr_clear', () => {
			stopAllHotkeys();
			overlaySession++;
			sessionDeltaBySlug = new Map();
			masteryReadSequence++;
			inventoryReadSequence++;
			words = [];
			processing = false;
			controlsEnabled = false;
			selectedIndex = null;
		}).then(registerCleanup);

		listen<{ action: string; pressed: boolean }>('overlay_hotkey', ({ payload }) =>
			onHotkeyEvent(payload.action, payload.pressed))
			.then(registerCleanup);
		listen('overlay_hotkey_reset', stopAllHotkeys).then(registerCleanup);

		listen('relic_reward_detected', () => {
			relicDetectionSound.currentTime = 0;
			void relicDetectionSound.play().catch((error) => {
				console.error('[relic detection] failed to play sound', error);
			});
		}).then(registerCleanup);

		listen<{ name: string }>('relic_reward_added', ({ payload }) => {
			const generation = ++relicFeedbackGeneration;
			clearTimeout(relicFeedbackTimer);
			relicFeedback = `Added +1 ${payload.name}`;
			void invoke('show_relic_add_toast').then(() => {
				if (generation !== relicFeedbackGeneration) return;
				relicFeedbackTimer = setTimeout(() => {
					if (generation === relicFeedbackGeneration) relicFeedback = null;
				}, 10000);
			}).catch((error) => {
				if (generation === relicFeedbackGeneration) relicFeedback = null;
				console.error('Could not show relic inventory confirmation:', error);
			});
		}).then(registerCleanup);

		return () => {
			disposed = true;
			clearTimeout(relicFeedbackTimer);
			stopAllHotkeys();
			masteryReadSequence++;
			inventoryReadSequence++;
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

{#snippet keycap(value: string)}
	<kbd class="inline-flex justify-center items-center bg-surface/90 px-1.5 border border-border-secondary min-w-5 h-5 font-mono font-semibold text-[10px] text-foreground tracking-wide">
		{value}
	</kbd>
{/snippet}

<main class="relative w-screen h-screen pointer-events-none">
	{#if relicFeedback}
		<div class="absolute bottom-4 left-1/2 -translate-x-1/2">
			<div
				in:flyAndScale={{ y: 28, duration: 400 }}
				out:fade={{ duration: 450 }}
				class="bg-background/95 px-4 py-2 border border-accent text-foreground text-sm shadow-lg whitespace-nowrap"
			>
				{relicFeedback}
			</div>
		</div>
	{/if}
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
	{#each words as word, index (`${word.text}-${word.x}-${word.y}-${word.width}-${word.height}`)}
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
				'selection-ring': selectedIndex === index,
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
				<div class="font-medium text-[10px] text-muted-foreground">
					{#if word.vaulted}
						<span class="text-amber-500">vaulted</span><span class="mx-0.5">•</span>
					{/if}
					{ownedCount} owned
					<span class="ml-0.5" class:text-accent={sessionDelta > 0} class:text-red-400={sessionDelta < 0}>
						({sessionDelta >= 0 ? '+' : ''}{sessionDelta})
					</span>
				</div>
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
	{/each}
	{#if controlsEnabled && !processing && words.length > 0}
		<aside
			aria-label="Overlay keyboard shortcuts"
			class="absolute right-4 bottom-4 bg-background/95 px-3 py-2 border border-border-secondary text-foreground text-xs shadow-lg whitespace-nowrap"
		>
			<div class="flex items-center gap-3">
				<div class="flex items-center gap-1.5">
					<Icon icon="material-symbols:autorenew-rounded" class="size-4 text-accent" aria-hidden="true" />
					<span class="text-muted-foreground">Cycle</span>
					{@render keycap(shortcut('cycle'))}<span>next</span>
					{@render keycap(shortcut('cycle_back'))}<span>back</span>
				</div>

				<div class="flex items-center gap-1 border-l border-border-secondary pl-3">
					<Icon icon="material-symbols:open-with-rounded" class="size-4 text-accent" aria-hidden="true" />
					<span class="mr-0.5 text-muted-foreground">Navigate</span>
					{@render keycap(shortcut('navigate_up'))}
					{@render keycap(shortcut('navigate_left'))}
					{@render keycap(shortcut('navigate_down'))}
					{@render keycap(shortcut('navigate_right'))}
				</div>

				<div class="flex items-center gap-1.5 border-l border-border-secondary pl-3">
					<Icon icon="material-symbols:inventory-2-outline-rounded" class="size-4 text-accent" aria-hidden="true" />
					<span class="text-muted-foreground">Inventory</span>
					{@render keycap(shortcut('inventory_decrement'))}<span>−1</span>
					{@render keycap(shortcut('inventory_increment'))}<span>+1</span>
					{@render keycap(shortcut('inventory_add_all'))}<span>all</span>
				</div>

				<div class="flex items-center gap-1.5 border-l border-border-secondary pl-3">
					<Icon icon="material-symbols:close-rounded" class="size-4 text-accent" aria-hidden="true" />
					<span class="text-muted-foreground">Dismiss</span>
					{@render keycap('ESC')}
				</div>
			</div>
		</aside>
	{/if}
</main>

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
		) border-box;
		background-size: 200% 100%;
		mask: linear-gradient(#fff 0 0) padding-box, linear-gradient(#fff 0 0);
		mask-composite: exclude;
		-webkit-mask: linear-gradient(#fff 0 0) padding-box, linear-gradient(#fff 0 0);
		-webkit-mask-composite: xor;
		animation: selection-border-flow 2s linear infinite;
	}

	@keyframes selection-border-flow {
		to { background-position: 100% 0; }
	}

	@media (prefers-reduced-motion: reduce) {
		.selection-ring::after { animation: none; }
	}
</style>
