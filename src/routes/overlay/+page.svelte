<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/core';
	import { LazyStore } from '@tauri-apps/plugin-store';
	import { onMount } from 'svelte';
	import { loadSettings, watchOverlayPriceSettings } from '$lib/settings.svelte';
	import { changeOcrItemQuantities, inventoryNameKey, type InventoryItem } from '$lib/inventory';
	import OverlayItem from './components/OverlayItem.svelte';
	import OverlayShortcuts from './components/OverlayShortcuts.svelte';
	import OverlayStatus from './components/OverlayStatus.svelte';
	import type { OcrWord } from './components/types';

	let words: OcrWord[] = $state([]);
	let showBoundingBoxes = $state(false);
	let processing = $state(false);
	let controlsEnabled = $state(false);
	let selectedIndex = $state<number | null>(null);
	let sessionDeltaBySlug = $state(new Map<string, number>());
	let overlaySession = 0;
	let inventoryChangeQueue: Promise<void> = Promise.resolve();
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
		'cycle',
		'cycle_back',
		'navigate_up',
		'navigate_left',
		'navigate_down',
		'navigate_right',
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
		const positioned = items
			.map((item, index) => ({
				index,
				x: item.x + item.width / 2,
				y: item.y + item.height / 2,
			}))
			.sort((a, b) => a.y - b.y || a.x - b.x);
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

	function queueInventoryChange(change: (session: number) => Promise<void>) {
		const session = overlaySession;
		inventoryChangeQueue = inventoryChangeQueue
			.catch(() => undefined)
			.then(() => session === overlaySession ? change(session) : undefined);
	}

	function applyInventoryChanges(changes: { word: OcrWord; delta: number }[]) {
		queueInventoryChange((session) => saveInventoryChanges(changes, session));
	}

	async function saveInventoryChanges(changes: { word: OcrWord; delta: number }[], session: number) {
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

	function resetSessionInventory() {
		queueInventoryChange(async (session) => {
			const wordsBySlug = new Map(words.filter((word) => word.slug).map((word) => [word.slug!, word]));
			const changes = [...sessionDeltaBySlug]
				.filter(([slug, delta]) => delta !== 0 && wordsBySlug.has(slug))
				.map(([slug, delta]) => ({ word: wordsBySlug.get(slug)!, delta: -delta }));
			if (changes.length > 0) await saveInventoryChanges(changes, session);
		});
	}

	function handleOverlayHotkey(action: string) {
		if (!controlsEnabled || processing || words.length === 0) return;
		if (action === 'cycle' || action === 'cycle_back') {
			const position = selectedIndex === null ? -1 : cycleOrder.indexOf(selectedIndex);
			selectedIndex =
				action === 'cycle_back'
					? cycleOrder[
							position < 0
								? cycleOrder.length - 1
								: (position - 1 + cycleOrder.length) % cycleOrder.length
						]
					: cycleOrder[(position + 1) % cycleOrder.length];
			return;
		}
		if (action.startsWith('navigate_')) {
			if (selectedIndex === null) {
				selectedIndex = cycleOrder[0];
				return;
			}
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
			const candidates = words
				.map((word, index) => ({
					index,
					dx: word.x + word.width / 2 - x,
					dy: word.y + word.height / 2 - y,
				}))
				.filter((item) => item.index !== selectedIndex);
			const ahead = candidates.filter((item) =>
				direction === 'left' ? item.dx < -1 : item.dx > 1,
			);
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
			applyInventoryChanges(
				words.map((word) => ({
					word,
					delta:
						word.quantity != null && Number.isSafeInteger(word.quantity) && word.quantity > 0
							? word.quantity
							: 1,
				})),
			);
			return;
		}
		if (action === 'inventory_reset_session') {
			resetSessionInventory();
			return;
		}
		if (selectedIndex === null) return;
		if (action === 'inventory_increment' || action === 'inventory_decrement') {
			applyInventoryChanges([
				{
					word: words[selectedIndex],
					delta: action === 'inventory_increment' ? 1 : -1,
				},
			]);
		}
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
			onHotkeyEvent(payload.action, payload.pressed),
		).then(registerCleanup);
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
			void invoke('show_relic_add_toast')
				.then(() => {
					if (generation !== relicFeedbackGeneration) return;
					relicFeedbackTimer = setTimeout(() => {
						if (generation === relicFeedbackGeneration) relicFeedback = null;
					}, 10000);
				})
				.catch((error) => {
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
</script>

<main class="relative w-screen h-screen pointer-events-none">
	<OverlayStatus {relicFeedback} {processing} />
	{#each words as word, index (`${word.text}-${word.x}-${word.y}-${word.width}-${word.height}`)}
		<OverlayItem
			{word}
			selected={selectedIndex === index}
			{showBoundingBoxes}
			{masteredSlugs}
			{ownedBySlug}
			{ownedByName}
			{sessionDeltaBySlug}
		/>
	{/each}
	{#if controlsEnabled && !processing && words.length > 0}
		<OverlayShortcuts />
	{/if}
</main>
