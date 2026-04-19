<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';

	import InventoryMain from '$lib/components/dashboard/InventoryMain.svelte';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import DashboardMain from './dashboard/Main.svelte';
	import MasteryMain from './mastery/Main.svelte';

	type OcrWord = {
		text: string;
		price?: string | null;
	};

	type OcrResultPayload = {
		words?: OcrWord[];
	};

	type InventoryItem = {
		name: string;
		quantity: number;
		medianPrice: string | null;
	};

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
		return name.replace(/^\*+/, '').split(/\s+/).filter(Boolean).join(' ').trim();
	}

	function normalizeMedianPrice(price: string | null | undefined): string | null {
		const normalized = price?.trim() ?? '';
		return normalized.length > 0 ? normalized : null;
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
				medianPrice?: unknown;
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

			const normalizedMedianPrice =
				typeof candidate.medianPrice === 'string'
					? normalizeMedianPrice(candidate.medianPrice)
					: null;

			const existing = mergedByName.get(normalizedName);
			if (!existing) {
				mergedByName.set(normalizedName, {
					name: normalizedName,
					quantity: numericQuantity,
					medianPrice: normalizedMedianPrice,
				});
				continue;
			}

			mergedByName.set(normalizedName, {
				name: normalizedName,
				quantity: existing.quantity + numericQuantity,
				medianPrice: normalizedMedianPrice ?? existing.medianPrice,
			});
		}

		return [...mergedByName.values()];
	}

	function isLowConfidenceDictionaryMatch(name: string): boolean {
		return /^\s*\*/.test(name);
	}

	function addDetectedInventoryItem(word: OcrWord) {
		if (isLowConfidenceDictionaryMatch(word.text)) {
			return;
		}

		const name = normalizeDetectedName(word.text);
		if (!name) {
			return;
		}

		const nextItems = [...inventoryItems];
		const index = nextItems.findIndex((item) => item.name === name);
		const detectedPrice = normalizeMedianPrice(word.price);

		if (index === -1) {
			nextItems.push({
				name,
				quantity: 1,
				medianPrice: detectedPrice,
			});
			inventoryItems = nextItems;
			return;
		}

		const existing = nextItems[index];
		nextItems[index] = {
			...existing,
			quantity: existing.quantity + 1,
			medianPrice: detectedPrice ?? existing.medianPrice,
		};
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

<Tabs.Root value="dashboard" class="w-full">
	<Tabs.List>
		<Tabs.Trigger value="dashboard">Dashboard</Tabs.Trigger>
		<Tabs.Trigger value="mastery">Mastery</Tabs.Trigger>
		<Tabs.Trigger value="inventory">Inventory</Tabs.Trigger>
	</Tabs.List>
	<Tabs.Content value="dashboard" class="mt-4">
		<DashboardMain />
	</Tabs.Content>
	<Tabs.Content value="mastery" class="mt-4">
		<MasteryMain />
	</Tabs.Content>
	<Tabs.Content value="inventory" class="mt-4">
		<InventoryMain
			items={sortedInventoryItems}
			onIncrease={incrementInventoryItem}
			onDecrease={decrementInventoryItem}
			{clearInventory}
		/>
	</Tabs.Content>
</Tabs.Root>
