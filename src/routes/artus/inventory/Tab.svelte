<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import type { ComponentProps } from 'svelte';

	import InventoryMain from './Main.svelte';

	type OcrWord = {
		text: string;
		market_median?: number;
		market_median_from_current_offers?: boolean;
	};

	type OcrResultPayload = {
		words?: OcrWord[];
		show_ocr_bounding_boxes?: boolean;
	};

	type InventoryItem = ComponentProps<typeof InventoryMain>['items'][number];

	const INVENTORY_STORAGE_KEY = 'artus.inventory.v1';

	let inventoryItems = $state<InventoryItem[]>([]);
	let hasLoadedPersistedInventory = $state(false);

	const sortedInventoryItems = $derived.by(() =>
		[...inventoryItems].sort(
			(left, right) => right.quantity - left.quantity || left.name.localeCompare(right.name),
		),
	);

	$effect(() => {
		if (!hasLoadedPersistedInventory || typeof window === 'undefined') {
			return;
		}

		persistInventoryItems(inventoryItems);
	});

	onMount(() => {
		let unlistenOcrResult: (() => void) | undefined;

		loadPersistedInventoryItems();
		hasLoadedPersistedInventory = true;

		listen<OcrResultPayload>('ocr_result', (event) => {
			for (const word of event.payload?.words ?? []) {
				addDetectedInventoryItem(word);
			}
		}).then((cleanup) => {
			unlistenOcrResult = cleanup;
		});

		return () => {
			unlistenOcrResult?.();
		};
	});

	function normalizeDetectedName(name: string): string {
		return name.split(/\s+/).filter(Boolean).join(' ').trim();
	}

	function normalizeDetectedMedian(value: unknown): number | undefined {
		return typeof value === 'number' && Number.isFinite(value) ? value : undefined;
	}

	function buildInventoryItem(
		name: string,
		quantity: number,
		marketMedian?: number,
		marketMedianUsesOfferFallback?: boolean,
	): InventoryItem {
		const item: InventoryItem = {
			name,
			quantity,
		};

		if (marketMedian !== undefined) {
			item.marketMedian = marketMedian;
			if (marketMedianUsesOfferFallback !== undefined) {
				item.marketMedianUsesOfferFallback = marketMedianUsesOfferFallback;
			}
		}

		return item;
	}

	function loadPersistedInventoryItems() {
		if (typeof window === 'undefined') {
			return;
		}

		try {
			const raw = window.localStorage.getItem(INVENTORY_STORAGE_KEY);
			if (!raw) {
				inventoryItems = [];
				return;
			}

			const parsed = JSON.parse(raw);
			inventoryItems = sanitizePersistedInventoryItems(parsed);
		} catch (error) {
			console.warn('failed to load persisted inventory', error);
			inventoryItems = [];
		}
	}

	function persistInventoryItems(items: InventoryItem[]) {
		if (typeof window === 'undefined') {
			return;
		}

		try {
			if (items.length === 0) {
				window.localStorage.removeItem(INVENTORY_STORAGE_KEY);
				return;
			}

			window.localStorage.setItem(INVENTORY_STORAGE_KEY, JSON.stringify(items));
		} catch (error) {
			console.warn('failed to persist inventory', error);
		}
	}

	function sanitizePersistedInventoryItems(value: unknown): InventoryItem[] {
		if (!Array.isArray(value)) {
			return [];
		}

		const mergedByName = new Map<string, InventoryItem>();

		for (const entry of value) {
			if (!entry || typeof entry !== 'object') {
				continue;
			}

			const candidate = entry as {
				name?: unknown;
				quantity?: unknown;
				marketMedian?: unknown;
				marketMedianUsesOfferFallback?: unknown;
			};

			const normalizedName =
				typeof candidate.name === 'string' ? normalizeDetectedName(candidate.name) : '';
			if (!normalizedName) {
				continue;
			}

			const numericQuantity =
				typeof candidate.quantity === 'number' ? Math.trunc(candidate.quantity) : Number.NaN;

			if (!Number.isFinite(numericQuantity) || numericQuantity <= 0) {
				continue;
			}

			const marketMedian = normalizeDetectedMedian(candidate.marketMedian);
			const marketMedianUsesOfferFallback =
				marketMedian !== undefined && typeof candidate.marketMedianUsesOfferFallback === 'boolean'
					? candidate.marketMedianUsesOfferFallback
					: undefined;

			const existing = mergedByName.get(normalizedName);
			if (!existing) {
				mergedByName.set(
					normalizedName,
					buildInventoryItem(
						normalizedName,
						numericQuantity,
						marketMedian,
						marketMedianUsesOfferFallback,
					),
				);
				continue;
			}

			const nextMarketMedian = existing.marketMedian ?? marketMedian;
			const nextMarketMedianUsesOfferFallback =
				nextMarketMedian === undefined
					? undefined
					: existing.marketMedian !== undefined
						? existing.marketMedianUsesOfferFallback
						: marketMedianUsesOfferFallback;

			mergedByName.set(
				normalizedName,
				buildInventoryItem(
					normalizedName,
					existing.quantity + numericQuantity,
					nextMarketMedian,
					nextMarketMedianUsesOfferFallback,
				),
			);
		}

		return [...mergedByName.values()];
	}

	function addDetectedInventoryItem(word: OcrWord) {
		const name = normalizeDetectedName(word.text);
		if (!name) {
			return;
		}

		const marketMedian = normalizeDetectedMedian(word.market_median);
		const marketMedianUsesOfferFallback =
			marketMedian !== undefined && typeof word.market_median_from_current_offers === 'boolean'
				? word.market_median_from_current_offers
				: undefined;

		const nextItems = [...inventoryItems];
		const index = nextItems.findIndex((item) => item.name === name);

		if (index === -1) {
			nextItems.push(buildInventoryItem(name, 1, marketMedian, marketMedianUsesOfferFallback));
			inventoryItems = nextItems;
			return;
		}

		const existing = nextItems[index];
		const nextMarketMedian = existing.marketMedian ?? marketMedian;
		const nextMarketMedianUsesOfferFallback =
			nextMarketMedian === undefined
				? undefined
				: existing.marketMedian !== undefined
					? existing.marketMedianUsesOfferFallback
					: marketMedianUsesOfferFallback;

		nextItems[index] = buildInventoryItem(
			name,
			existing.quantity + 1,
			nextMarketMedian,
			nextMarketMedianUsesOfferFallback,
		);
		inventoryItems = nextItems;
	}

	function incrementInventoryItem(name: string) {
		changeInventoryItemQuantity(name, 1);
	}

	function decrementInventoryItem(name: string) {
		changeInventoryItemQuantity(name, -1);
	}

	function clearInventory() {
		inventoryItems = [];
	}

	function changeInventoryItemQuantity(name: string, delta: number) {
		const nextItems = [...inventoryItems];
		const index = nextItems.findIndex((item) => item.name === name);
		if (index === -1) {
			return;
		}

		const existing = nextItems[index];
		const nextQuantity = existing.quantity + delta;
		if (nextQuantity <= 0) {
			nextItems.splice(index, 1);
			inventoryItems = nextItems;
			return;
		}

		nextItems[index] = {
			...existing,
			quantity: nextQuantity,
		};
		inventoryItems = nextItems;
	}
</script>

<InventoryMain
	items={sortedInventoryItems}
	onIncrease={incrementInventoryItem}
	onDecrease={decrementInventoryItem}
	{clearInventory}
/>
