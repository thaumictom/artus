<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/core';
	import { LazyStore } from '@tauri-apps/plugin-store';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import { flyAndScale } from '$lib/transition';
	import { config, loadSettings, watchOverlayPriceSettings } from '$lib/settings.svelte';
	import { quicklistPrice } from '$lib/quicklist';
	import { GetOrdersResponseSchema } from '$lib/schemas';
	import { fetchMarketListings } from '$lib/market-listings';
	import { changeOcrItemQuantities, getInventorySnapshot, watchInventory, inventoryMarketSlug, inventoryNameKey, setOcrItemQuantities, type InventoryItem, type InventorySnapshot } from '$lib/inventory';
	import CreateListing from '../artus/inventory/CreateListing.svelte';
	import type { EditableListing, Listing, ListingChange } from '../artus/listings/types';
	import OverlayItem from './components/OverlayItem.svelte';
	import OverlayShortcuts from './components/OverlayShortcuts.svelte';
	import OverlayStatus from './components/OverlayStatus.svelte';
	import type { OcrWord } from './components/types';

	let words: OcrWord[] = $state([]);
	let showBoundingBoxes = $state(false);
	let processing = $state(false);
	let controlsEnabled = $state(false);
	let selectedIndex = $state<number | null>(null);
	let listingItem = $state<InventoryItem | null>(null);
	let listingEditing = $state<EditableListing | null>(null);
	let listingSubtype = $state<string | undefined>(undefined);
	let listingLookupStatus = $state<string | null>(null);
	let listingLookupLoading = $state(false);
	let listingLookupSequence = 0;
	let listingHotkey = $state<{ action: string; sequence: number } | null>(null);
	let listingHotkeySequence = 0;
	let listingDialogRegistered = false;
	let listingDialogQueue: Promise<void> = Promise.resolve();
	let marketLoggedIn = $state(false);
	let marketAuthRevision = 0;
	let sessionDeltaBySlug = $state(new Map<string, number>());
	let overlaySession = 0;
	let inventoryChangeQueue: Promise<void> = Promise.resolve();
	let relicFeedback = $state<string | null>(null);
	let relicFeedbackGeneration = 0;
	let masteredSlugs = $state(new Set<string>());
	let ownedBySlug = $state(new Map<string, number>());
	let ownedByName = $state(new Map<string, number>());
	let listingBySlug = $state(new Map<string, { status: 'active' | 'hidden'; platinum: number }>());
	let activeListingReadSequence = 0;
	let activeListingsReady = $state(false);
	let captureListings = $state<Listing[]>([]);
	let marketItemsBySlug = $state<Record<string, { id: string; maxRank?: number | null }>>({});
	let activeListingsPromise: Promise<void> = Promise.resolve();
	let quicklistBusy = $state(false);
	let quicklistFeedback = $state<{ message: string; error: boolean } | null>(null);
	let quicklistFeedbackTimer: ReturnType<typeof setTimeout> | undefined;
	let quicklistRequest = 0;
	const masteryStore = new LazyStore('mastery.json');
	let inventoryRevision = -1;
	let masteryReadSequence = 0;
	let inventoryReadSequence = 0;
	const visualRows = $derived(groupVisualRows(words));
	const cycleOrder = $derived(visualRows.flat());
	type HeldHotkey = {
		delay: ReturnType<typeof setTimeout>;
		interval?: ReturnType<typeof setInterval>;
	};
	const heldHotkeys = new Map<string, HeldHotkey>();
	type OcrMarketItem = { id: string; maxRank?: number | null };

	function listingForSlug(slug: string): { status: 'active' | 'hidden'; platinum: number } | null {
		const item = marketItemsBySlug[slug];
		if (!item) return null;
		const word = words.find((candidate) => candidate.slug === slug);
		const candidates = captureListings.filter((order) => order.type === 'sell' && order.itemId === item.id);
		const matching = candidates.find((order) =>
			(!word?.subtype || order.subtype?.toLowerCase() === word.subtype.toLowerCase()) &&
			(!item.maxRank || (order.rank ?? 0) === 0),
		) ?? candidates[0];
		return matching ? { status: matching.visible ? 'active' : 'hidden', platinum: matching.platinum } : null;
	}

	function updateListingForSlug(slug: string) {
		const next = new Map(listingBySlug);
		const targets = [slug, ...words.flatMap((word) => word.slug &&
			inventoryMarketSlug({ name: word.text, slug: word.slug, quantity: 0 }) === slug
			? [word.slug] : [])];
		for (const target of new Set(targets)) {
			const listing = listingForSlug(target);
			if (listing) next.set(target, listing);
			else next.delete(target);
		}
		listingBySlug = next;
	}

	function applyCaptureListingChange(change: ListingChange) {
		switch (change.kind) {
			case 'created':
				if (change.listing) {
					captureListings = [...captureListings, change.listing];
					updateListingForSlug(change.slug);
				} else {
					const next = new Map(listingBySlug);
					for (const word of words) {
						if (word.slug && inventoryMarketSlug({ name: word.text, slug: word.slug, quantity: 0 }) === change.slug)
							next.set(word.slug, { status: change.visible ? 'active' : 'hidden', platinum: change.platinum });
					}
					listingBySlug = next;
				}
				break;
			case 'updated':
				captureListings = captureListings.map((order) => order.id === change.id
					? { ...order, platinum: change.platinum, quantity: change.quantity } : order);
				if (change.slug) updateListingForSlug(change.slug);
				break;
			case 'visibility':
				captureListings = captureListings.map((order) => order.id === change.id
					? { ...order, visible: change.visible } : order);
				if (change.slug) updateListingForSlug(change.slug);
				break;
			case 'deleted':
				captureListings = captureListings.filter((order) => order.id !== change.id);
				if (change.slug) updateListingForSlug(change.slug);
				break;
		}
	}

	async function refreshActiveListings(slugs: string[], sequence: number, loggedIn = marketLoggedIn) {
		activeListingsReady = false;
		listingBySlug = new Map();
		captureListings = [];
		marketItemsBySlug = {};
		if (!loggedIn || slugs.length === 0) {
			return;
		}
		try {
			const [orders, marketItems] = await Promise.all([
				fetchMarketListings(),
				invoke<Record<string, OcrMarketItem>>('get_ocr_market_items', { slugs }),
			]);
			if (sequence !== activeListingReadSequence) return;
			captureListings = orders;
			marketItemsBySlug = marketItems;
			const itemListings = slugs.map((slug) => {
				const listing = listingForSlug(slug);
				return listing ? [slug, listing] as const : null;
			});
			if (sequence === activeListingReadSequence) {
				listingBySlug = new Map(itemListings.filter((entry): entry is readonly [string, { status: 'active' | 'hidden'; platinum: number }] => entry !== null));
				activeListingsReady = true;
			}
		} catch (error) {
			if (sequence === activeListingReadSequence) console.error('Could not read active overlay listings:', error);
		}
	}

	function showQuicklistFeedback(message: string, error = false) {
		clearTimeout(quicklistFeedbackTimer);
		quicklistFeedback = { message, error };
		quicklistFeedbackTimer = setTimeout(() => quicklistFeedback = null, 4000);
	}

	async function createQuicklist(word: OcrWord, slug: string) {
		if (quicklistBusy) return;
		quicklistBusy = true;
		const request = ++quicklistRequest;
		const session = overlaySession;
		const selected = selectedIndex;
		try {
			const [itemResponse, ordersResponse, statistics] = await Promise.all([
				invoke<{ data: { id: string; maxRank?: number; maxCharges?: number; maxAmberStars?: number; maxCyanStars?: number; subtypes?: string[] } }>('get_cached_wfm_item', { slug }),
				invoke('get_market_orders', { slug }),
				invoke<{ median: number | null } | null>('get_tradeable_today_statistics', { slug }),
			]);
			if (request !== quicklistRequest || session !== overlaySession || selected !== selectedIndex || !marketLoggedIn) return;
			const item = itemResponse.data;
			if (!item?.id) throw new Error('Could not read market item');
			const subtype = item.subtypes?.find((option) => option.toLowerCase() === word.subtype?.toLowerCase()) ?? item.subtypes?.[0] ?? null;
			if (captureListings.some((listing) => listing.type === 'sell' && listing.itemId === item.id &&
				(!subtype || listing.subtype?.toLowerCase() === subtype.toLowerCase()) &&
				(!item.maxRank || (listing.rank ?? 0) === 0))) {
				throw new Error('This item already has a listing');
			}
			const offers = GetOrdersResponseSchema.parse(ordersResponse).data
				.filter((order) => order.type === 'sell' && order.user.status === 'ingame' && order.quantity > 0 &&
					(!subtype || order.subtype?.toLowerCase() === subtype.toLowerCase()) &&
					(!item.maxRank || (order.rank ?? 0) === 0))
				.map((order) => order.platinum);
			const price = quicklistPrice(config.quicklist_price_strategy, statistics?.median ?? word.market_median ?? null, offers);
			const quantity = Math.min(9999, Math.max(1, Math.floor(ownedBySlug.get(slug) ?? ownedByName.get(inventoryNameKey(word.text)) ?? word.quantity ?? 1)));
			const created = await invoke<{ data?: Listing }>('market_create_listing', {
				slug, platinum: price, quantity, visible: !config.quicklist_hide_first,
				variant: {
					rank: item.maxRank ? 0 : null, charges: item.maxCharges ? 0 : null,
					amberStars: item.maxAmberStars ? 0 : null, cyanStars: item.maxCyanStars ? 0 : null,
					subtype,
				},
			});
			if (request === quicklistRequest && session === overlaySession) {
				const listing = created.data;
				applyCaptureListingChange({ kind: 'created', slug, platinum: price, visible: !config.quicklist_hide_first,
					listing: listing && typeof listing.id === 'string' && typeof listing.itemId === 'string' ? listing : null });
				showQuicklistFeedback(`Listing created for ${price} platinum${config.quicklist_hide_first ? ' (hidden)' : ''}`);
			}
		} catch (error) {
			if (request === quicklistRequest && session === overlaySession) showQuicklistFeedback(`Could not quicklist: ${String(error)}`, true);
		} finally {
			quicklistBusy = false;
		}
	}

	$effect(() => {
		const sequence = ++activeListingReadSequence;
		const slugs = [...new Set(words.flatMap((word) => word.slug ? [word.slug] : []))];
		activeListingsPromise = refreshActiveListings(slugs, sequence, marketLoggedIn);
	});
	const repeatableActions = new Set([
		'cycle',
		'cycle_back',
		'navigate_up',
		'navigate_left',
		'navigate_down',
		'navigate_right',
	]);

	$effect(() => {
		const open = listingItem !== null;
		if (open === listingDialogRegistered) return;
		listingDialogRegistered = open;
		// Keep backend shortcut state in the same order as rapid open/close changes.
		listingDialogQueue = listingDialogQueue.then(async () => {
			if (open && listingItem === null) return;
			try {
				await invoke('set_overlay_listing_dialog_open', { open });
			} catch (error) {
				console.error('Could not update overlay listing shortcuts:', error);
				if (open) listingItem = null;
			}
		});
	});

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

	function closeListing() {
		listingItem = null;
		listingEditing = null;
		listingHotkey = null;
	}

	function cancelListingLookup() {
		listingLookupSequence++;
		listingLookupLoading = false;
		listingLookupStatus = null;
	}

	async function openListingFor(word: OcrWord, slug: string) {
		const request = ++listingLookupSequence;
		listingLookupLoading = true;
		listingLookupStatus = 'Checking your listings…';
		try {
			await activeListingsPromise;
			if (!activeListingsReady) throw new Error('Listings are not available yet');
			const itemResponse = await invoke<{ data: { id: string; maxRank?: number; subtypes?: string[] } }>('get_cached_wfm_item', { slug });
			if (request !== listingLookupSequence) return;
			const item = itemResponse.data;
			if (typeof item?.id !== 'string') {
				throw new Error('Could not read your market listings');
			}
			const subtype = word.subtype ?? item.subtypes?.[0];
			const sellListings = captureListings.filter((order) =>
				order.type === 'sell' && order.itemId === item.id,
			);
			const existing = sellListings.find((order) =>
				(!subtype || order.subtype?.toLowerCase() === subtype.toLowerCase()) &&
				(!item.maxRank || (order.rank ?? 0) === 0),
			) ?? sellListings[0];
			listingSubtype = subtype;
			listingEditing = existing ?? null;
			listingHotkey = null;
			listingItem = {
				name: word.text,
				slug,
				quantity: ownedBySlug.get(word.slug ?? '') ?? ownedByName.get(inventoryNameKey(word.text)) ?? word.quantity ?? 0,
			};
			listingLookupStatus = null;
		} catch (error) {
			if (request !== listingLookupSequence) return;
			console.error('Could not check existing market listings:', error);
			listingLookupStatus = `Could not check listings: ${String(error)}`;
		} finally {
			if (request === listingLookupSequence) listingLookupLoading = false;
		}
	}

	function onHotkeyEvent(action: string, pressed: boolean) {
		if (action === 'listing_cancel' && pressed && listingItem) {
			dispatchHotkey(action);
			return;
		}
		if (!controlsEnabled) return;
		if (!pressed) {
			stopHotkey(action);
			return;
		}
		if (heldHotkeys.has(action)) return;
		dispatchHotkey(action);
		const held: HeldHotkey = {
			delay: setTimeout(() => {
				if (heldHotkeys.get(action) !== held) return;
				held.interval = setInterval(() => dispatchHotkey(action), 85);
			}, 200),
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
			.then(() => (session === overlaySession ? change(session) : undefined));
	}

	function applyInventoryChanges(changes: { word: OcrWord; delta: number }[]) {
		queueInventoryChange((session) => saveInventoryChanges(changes, session));
	}

	function dispatchHotkey(action: string) {
		if (action === 'listing_cancel') {
			closeListing();
			stopAllHotkeys();
			return;
		}
		if (listingItem) {
			listingHotkey = { action, sequence: ++listingHotkeySequence };
			return;
		}
		handleOverlayHotkey(action);
	}

	function syncScannedQuantities(scannedWords: OcrWord[]) {
		const quantities = new Map<string, { word: OcrWord; quantity: number }>();
		for (const word of scannedWords) {
			if (!word.slug || word.is_custom) continue;
			const quantity = word.quantity ?? 1;
			if (!Number.isSafeInteger(quantity) || quantity <= 0) continue;
			quantities.set(word.slug, { word, quantity });
		}
		if (quantities.size === 0) return;
		queueInventoryChange(async (session) => {
			try {
				const applied = await setOcrItemQuantities([...quantities.values()]);
				if (session !== overlaySession) return;
				sessionDeltaBySlug = new Map(applied);
			} catch (error) {
				console.error('Could not sync scanned inventory quantities:', error);
			}
		});
	}

	async function saveInventoryChanges(
		changes: { word: OcrWord; delta: number }[],
		session: number,
	) {
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
		if (action === 'quicklist') {
			if (!marketLoggedIn || !activeListingsReady || quicklistBusy || selectedIndex === null) return;
			const word = words[selectedIndex];
			if (!word.slug || word.is_custom || listingBySlug.has(word.slug)) return;
			const slug = inventoryMarketSlug({ name: word.text, slug: word.slug, quantity: 0 });
			if (slug) void createQuicklist(word, slug);
			return;
		}
		if (action === 'create_sell_listing') {
			if (!marketLoggedIn) return;
			if (listingLookupLoading) return;
			if (selectedIndex === null) return;
			const word = words[selectedIndex];
			if (!word.slug || word.is_custom) return;
			const slug = inventoryMarketSlug({ name: word.text, slug: word.slug, quantity: 0 });
			if (!slug) return;
			void openListingFor(word, slug);
			return;
		}
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
		if (selectedIndex === null) return;
		if (action === 'inventory_increment' || action === 'inventory_decrement') {
			if (words[selectedIndex].is_custom) return;
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
			const snapshot = await getInventorySnapshot();
			if (sequence === inventoryReadSequence) applyInventorySnapshot(snapshot);
		} catch (error) {
			console.error('Could not read overlay inventory:', error);
		}
	}
	function applyInventorySnapshot(snapshot: InventorySnapshot) {
		if (snapshot.revision < inventoryRevision) return;
		inventoryRevision = snapshot.revision;
		updateOwnedCounts(snapshot.items);
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
		listen<boolean>('market_auth_changed', ({ payload }) => {
			marketAuthRevision++;
			marketLoggedIn = payload;
			if (!payload) {
				cancelListingLookup();
				closeListing();
			}
		}).then((cleanup) => {
			registerCleanup(cleanup);
			const authRevision = marketAuthRevision;
			void invoke<boolean>('market_authenticated')
				.then((authenticated) => {
					if (!disposed && authRevision === marketAuthRevision) marketLoggedIn = authenticated;
				})
				.catch((error) => console.error('Could not read market session state:', error));
		});
		void refreshMasteredSlugs(masteryReadSequence);
		watchInventory((snapshot) => {
			if (!disposed) applyInventorySnapshot(snapshot);
		}).then((stop) => {
			registerCleanup(stop);
			if (!disposed) void refreshInventory(inventoryReadSequence);
		}).catch((error) => {
			console.error('Could not observe overlay inventory:', error);
			if (!disposed) void refreshInventory(inventoryReadSequence);
		});

		listen('ocr_processing', () => {
			cancelListingLookup();
			closeListing();
			relicFeedbackGeneration++;
			clearTimeout(relicFeedbackTimer);
			clearTimeout(quicklistFeedbackTimer);
			relicFeedback = null;
			quicklistFeedback = null;
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
				cancelListingLookup();
				closeListing();
				stopAllHotkeys();
				overlaySession++;
				sessionDeltaBySlug = new Map();
				processing = false;
				controlsEnabled = event.payload?.controls_enabled ?? false;
				words = event.payload?.words ?? [];
				if (controlsEnabled) syncScannedQuantities(words);
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
			clearTimeout(quicklistFeedbackTimer);
			quicklistFeedback = null;
			cancelListingLookup();
			closeListing();
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
			cancelListingLookup();
			if (listingDialogRegistered) {
				void listingDialogQueue.then(() => invoke('set_overlay_listing_dialog_open', { open: false }));
			}
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
	<OverlayStatus {relicFeedback} {processing} {quicklistFeedback} listingStatus={listingLookupStatus} />
	{#each words as word, index (`${word.text}-${word.x}-${word.y}-${word.width}-${word.height}`)}
		<div class="absolute inset-0" in:flyAndScale={{ y: 24 }} out:fade={{ duration: 100 }}>
			<OverlayItem
				{word}
				selected={selectedIndex === index}
				{showBoundingBoxes}
				{masteredSlugs}
				{ownedBySlug}
				{ownedByName}
				{sessionDeltaBySlug}
				{listingBySlug}
			/>
		</div>
	{/each}
	{#if controlsEnabled && !processing && words.length > 0 && !listingItem}
		<OverlayShortcuts {marketLoggedIn} quicklistAvailable={marketLoggedIn && activeListingsReady && selectedIndex !== null && !!words[selectedIndex]?.slug && !words[selectedIndex]?.is_custom && !listingBySlug.has(words[selectedIndex].slug!)} />
	{/if}
	<CreateListing
		bind:item={() => listingItem, (value) => { listingItem = value; if (value === null) listingEditing = null; }}
		editing={listingEditing}
		overlayMode
		overlayHotkey={listingHotkey}
		initialSubtype={listingSubtype}
		onSaved={applyCaptureListingChange}
	/>
</main>
