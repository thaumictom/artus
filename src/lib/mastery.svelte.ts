import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { LazyStore } from '@tauri-apps/plugin-store';
import { z } from 'zod';
import { DictionarySchema } from '$lib/schemas';

const catalogItemSchema = z.object({
	name: z.string(),
	masterable: z.boolean().optional(),
	category: z.string().nullish(),
	type: z.string().nullish(),
	masteryReq: z.number().nullish(),
	itemCount: z.number().int().nullish(),
	components: z.array(z.string()).nullish(),
	tags: z.array(z.string()).nullish(),
	marketSlug: z.string().nullish(),
	wikiaUrl: z.string().nullish(),
	tradable: z.boolean().optional(),
	ducats: z.number().nullish(),
});

const tradeablePriceSchema = z.record(z.string(), z.object({
	median: z.number(),
	from_current_offers: z.boolean(),
}));

export type MasteryItem = Omit<z.infer<typeof catalogItemSchema>, 'components'> & {
	key: string;
	components: MasteryItem[];
};

const store = new LazyStore('mastery.json');
export const mastery = $state({
	items: [] as MasteryItem[],
	checked: [] as string[],
	automatic: [] as string[],
	otherXp: 0,
	prices: {} as z.infer<typeof tradeablePriceSchema>,
	ducats: {} as Record<string, number>,
	loading: true,
	error: ''
});

let startPromise: Promise<void> | null = null;
let unlisten: UnlistenFn | undefined;
let saveQueue = Promise.resolve();

function persist() {
	const checked = [...mastery.checked];
	const automatic = [...mastery.automatic];
	const otherXp = mastery.otherXp;
	saveQueue = saveQueue.then(async () => {
		await store.set('checked', checked);
		await store.set('automatic', automatic);
		await store.set('otherXp', otherXp);
		await store.save();
	}).catch((error) => console.error('Could not save mastery progress:', error));
}

export function setMasteryChecked(key: string, checked: boolean) {
	mastery.checked = checked
		? [...new Set([...mastery.checked, key])]
		: mastery.checked.filter((value) => value !== key);
	mastery.automatic = mastery.automatic.filter((value) => value !== key);
	persist();
}

export function dismissMasteryDots() {
	mastery.automatic = [];
	persist();
}

export function resetMasteryItems() {
	mastery.checked = [];
	mastery.automatic = [];
	persist();
}

export function setOtherMasteryXp(xp: number) {
	if (!Number.isSafeInteger(xp) || xp < 0) return;
	mastery.otherXp = xp;
	persist();
}

function markOcrWords(words: { mastery_key?: string }[]) {
	const validKeys = new Set(mastery.items.flatMap((item) => [item.key, ...item.components.map((part) => part.key)]));
	const found = new Set(words.map((word) => word.mastery_key).filter((key): key is string => !!key && validKeys.has(key)));
	if (found.size === 0) return;
	mastery.checked = [...new Set([...mastery.checked, ...found])];
	mastery.automatic = [...new Set([...mastery.automatic, ...found])];
	persist();
}

export function initializeMastery() {
	if (startPromise) return startPromise;
	startPromise = (async () => {
		try {
			const [checked, automatic, otherXp, response] = await Promise.all([
				store.get<string[]>('checked'),
				store.get<string[]>('automatic'),
				store.get<number>('otherXp'),
				invoke<Record<string, unknown>>('get_cached_market_items')
			]);
			mastery.checked = Array.isArray(checked) ? checked : [];
			mastery.automatic = Array.isArray(automatic) ? automatic : [];
			mastery.otherXp = typeof otherXp === 'number' && Number.isSafeInteger(otherXp) && otherXp >= 0 ? otherXp : 0;
			const catalog = new Map<string, z.infer<typeof catalogItemSchema>>();
			for (const [key, value] of Object.entries(response)) {
				const parsed = catalogItemSchema.safeParse(value);
				if (parsed.success) catalog.set(key, parsed.data);
			}
			const catalogDucats: Record<string, number> = {};
			for (const item of catalog.values()) {
				if (item.marketSlug && item.ducats != null) catalogDucats[item.marketSlug] = item.ducats;
			}
			mastery.ducats = catalogDucats;
			mastery.items = [...catalog]
				.filter(([, item]) => item.masterable)
				.map(([key, item]) => ({
					...item,
					key,
					components: (item.components ?? []).flatMap((componentKey) => {
						const component = catalog.get(componentKey);
						return component ? [{ ...component, key: componentKey, components: [] }] : [];
					})
				}))
				.sort((a, b) => a.name.localeCompare(b.name));
			void invoke('get_mastery_tradeable_prices')
				.then((value) => { mastery.prices = tradeablePriceSchema.parse(value); })
				.catch((error) => console.error('Could not load mastery median prices:', error));
			void invoke('get_market_dictionary')
				.then((value) => {
					const dictionary = DictionarySchema.parse(value);
					const ducats = { ...catalogDucats };
					for (const item of dictionary.items) {
						if (item.ducats != null) ducats[item.slug] = item.ducats;
					}
					mastery.ducats = ducats;
				})
				.catch((error) => console.error('Could not load mastery ducat values:', error));
			unlisten = await listen<{ words: { mastery_key?: string }[]; is_mastery_add?: boolean }>('ocr_result', (event) => {
				if (event.payload.is_mastery_add) markOcrWords(event.payload.words);
			});
			mastery.error = '';
		} catch (error) {
			mastery.error = 'Could not load mastery items. Restart Artus after the item catalog has loaded.';
			startPromise = null;
			console.error('Could not initialize mastery:', error);
		} finally {
			mastery.loading = false;
		}
	})();
	return startPromise;
}

export function stopMasteryListener() {
	unlisten?.();
	unlisten = undefined;
	startPromise = null;
}
