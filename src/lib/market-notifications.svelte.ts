import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { LazyStore } from '@tauri-apps/plugin-store';
import { DictionarySchema, GetItemResponseSchema, OrderSchema } from '$lib/schemas';
import { publishNotification } from '$lib/notifications.svelte';

export type MarketNotificationIntent = 'buy' | 'sell';
export type MarketNotificationDuration = 'session' | '1h' | '6h' | '24h' | '7d' | 'unlimited';

export type MarketNotificationRule = {
	id: string;
	itemId: string;
	slug: string;
	itemName: string;
	intent: MarketNotificationIntent;
	price: number;
	createdAt: number;
	expiresAt: number | null;
	sessionOnly: boolean;
	notifyOnce?: boolean;
};

export type MarketNotificationItem = {
	label: string;
	value: string;
};

type ConnectionState = 'disconnected' | 'connecting' | 'connected';
type MarketSocketState = {
	state: ConnectionState;
	error?: string | null;
};

const STORE_KEY = 'rules';
const store = new LazyStore('market-notifications.json');

let initialization: Promise<void> | null = null;
let itemLoad: Promise<void> | null = null;
let saveQueue = Promise.resolve();
let expiryTimer: ReturnType<typeof setTimeout> | undefined;
let socketListeners: Promise<void> | null = null;
let socketRequested = false;
const seenOrderIds = new Set<string>();
const seenOrderQueue: string[] = [];

export const marketNotificationState = $state({
	rules: [] as MarketNotificationRule[],
	items: [] as MarketNotificationItem[],
	itemsLoading: false,
	itemsError: null as string | null,
	connection: 'disconnected' as ConnectionState,
	connectionError: null as string | null,
});

export function initializeMarketNotifications() {
	if (initialization) return initialization;
	initialization = (async () => {
		try {
			await initializeSocketListeners();
			const saved = await store.get<MarketNotificationRule[]>(STORE_KEY);
			if (Array.isArray(saved)) {
				marketNotificationState.rules = saved.filter(isStoredRule).filter(isRuleActive);
			}
		} catch (error) {
			console.error('Could not load market notification rules:', error);
		} finally {
			scheduleExpiryCleanup();
			syncSocket();
		}
	})();
	return initialization;
}

export function loadMarketNotificationItems() {
	if (marketNotificationState.items.length > 0) return Promise.resolve();
	if (itemLoad) return itemLoad;
	marketNotificationState.itemsLoading = true;
	marketNotificationState.itemsError = null;
	itemLoad = invoke('get_market_dictionary')
		.then((response) => {
			const dictionary = DictionarySchema.parse(response);
			marketNotificationState.items = dictionary.items
				.map((item) => ({ label: item.name, value: item.slug }))
				.sort((a, b) => a.label.localeCompare(b.label));
		})
		.catch((error) => {
			console.error('Could not load market notification items:', error);
			marketNotificationState.itemsError = 'Could not load the market item list.';
			throw error;
		})
		.finally(() => {
			marketNotificationState.itemsLoading = false;
			itemLoad = null;
		});
	return itemLoad;
}

export async function addMarketNotificationRule(input: {
	slug: string;
	itemName: string;
	intent: MarketNotificationIntent;
	price: number;
	duration: MarketNotificationDuration;
	notifyOnce: boolean;
}) {
	const response = await invoke('get_market_item', { slug: input.slug });
	const item = GetItemResponseSchema.parse(response).data;

	const now = Date.now();
	const rule: MarketNotificationRule = {
		id: globalThis.crypto?.randomUUID?.() ?? `${now}-${Math.random().toString(36).slice(2)}`,
		itemId: item.id,
		slug: input.slug,
		itemName: input.itemName,
		intent: input.intent,
		price: input.price,
		createdAt: now,
		expiresAt: durationExpiry(input.duration, now),
		sessionOnly: input.duration === 'session',
		notifyOnce: input.notifyOnce,
	};
	marketNotificationState.rules = [rule, ...marketNotificationState.rules];
	await persistRules();
	scheduleExpiryCleanup();
	syncSocket();
	return rule;
}

export async function removeMarketNotificationRule(id: string) {
	marketNotificationState.rules = marketNotificationState.rules.filter((rule) => rule.id !== id);
	await persistRules();
	scheduleExpiryCleanup();
	syncSocket();
}

export function activeMarketNotificationCount() {
	return marketNotificationState.rules.filter(isRuleActive).length;
}

function syncSocket() {
	const shouldRun = activeMarketNotificationCount() > 0;
	if (shouldRun === socketRequested) return;
	socketRequested = shouldRun;

	if (!shouldRun) {
		marketNotificationState.connection = 'disconnected';
		marketNotificationState.connectionError = null;
		void invoke('stop_market_notification_socket').catch((error) =>
			console.error('Could not stop the market notification socket:', error),
		);
		return;
	}
	marketNotificationState.connection = 'connecting';
	marketNotificationState.connectionError = null;
	void invoke('start_market_notification_socket').catch((error) => {
		marketNotificationState.connection = 'disconnected';
		marketNotificationState.connectionError = String(error);
	});
}

function initializeSocketListeners() {
	if (socketListeners) return socketListeners;
	socketListeners = Promise.all([
		listen<unknown>('market_order', (event) => void handleNewOrder(event.payload)),
		listen<MarketSocketState>('market_socket_state', ({ payload }) => {
			marketNotificationState.connection = payload.state;
			marketNotificationState.connectionError = payload.error ?? null;
		}),
	]).then(() => undefined);
	return socketListeners;
}

async function handleNewOrder(payload: unknown) {
	const parsed = OrderSchema.safeParse(payload);
	if (!parsed.success) {
		console.error('Invalid order received from Warframe Market:', parsed.error);
		return;
	}
	const order = parsed.data;
	if (!order.visible || seenOrderIds.has(order.id)) return;
	rememberOrder(order.id);

	const unitPrice = order.platinum / Math.max(1, order.perTrade ?? 1);
	for (const rule of marketNotificationState.rules.filter(isRuleActive)) {
		if (rule.itemId !== order.itemId) continue;
		const matches =
			rule.intent === 'sell'
				? order.type === 'buy' && unitPrice >= rule.price
				: order.type === 'sell' && unitPrice <= rule.price;
		if (!matches) continue;

		// Remove one-shot rules from memory before awaiting notification delivery,
		// so another order event cannot trigger the same rule concurrently.
		let removal: Promise<void> | undefined;
		if (rule.notifyOnce !== false) {
			marketNotificationState.rules = marketNotificationState.rules.filter(
				(candidate) => candidate.id !== rule.id,
			);
			removal = persistRules();
			scheduleExpiryCleanup();
			syncSocket();
		}

		await publishNotification({
			source: 'market',
			sourceId: `${rule.id}:${order.id}`,
			title: `${rule.itemName} reached ${unitPrice.toLocaleString()} platinum`,
			market: {
				slug: rule.slug,
				orderType: order.type,
				since: Date.parse(order.createdAt) || Date.now(),
			},
			body:
				rule.intent === 'sell'
					? `New buy order meets your minimum of ${rule.price.toLocaleString()} platinum.`
					: `New sell order meets your maximum of ${rule.price.toLocaleString()} platinum.`,
		});
		await removal;
	}
}

function rememberOrder(id: string) {
	seenOrderIds.add(id);
	seenOrderQueue.push(id);
	if (seenOrderQueue.length <= 1_000) return;
	const oldest = seenOrderQueue.shift();
	if (oldest) seenOrderIds.delete(oldest);
}

function scheduleExpiryCleanup() {
	clearTimeout(expiryTimer);
	const now = Date.now();
	const expiredIds = new Set(
		marketNotificationState.rules
			.filter((rule) => rule.expiresAt !== null && rule.expiresAt <= now)
			.map((rule) => rule.id),
	);
	if (expiredIds.size > 0) {
		marketNotificationState.rules = marketNotificationState.rules.filter(
			(rule) => !expiredIds.has(rule.id),
		);
		void persistRules();
	}
	const nextExpiry = marketNotificationState.rules
		.map((rule) => rule.expiresAt)
		.filter((expiry): expiry is number => expiry !== null && expiry > now)
		.sort((a, b) => a - b)[0];
	if (nextExpiry !== undefined) {
		expiryTimer = setTimeout(() => {
			scheduleExpiryCleanup();
			syncSocket();
		}, Math.min(nextExpiry - now, 2_147_483_647));
	}
}

function persistRules() {
	const persistentRules = marketNotificationState.rules.filter((rule) => !rule.sessionOnly);
	saveQueue = saveQueue
		.catch(() => undefined)
		.then(async () => {
			await store.set(STORE_KEY, $state.snapshot(persistentRules));
			await store.save();
		})
		.catch((error) => console.error('Could not save market notification rules:', error));
	return saveQueue;
}

function isRuleActive(rule: MarketNotificationRule) {
	return rule.expiresAt === null || rule.expiresAt > Date.now();
}

function durationExpiry(duration: MarketNotificationDuration, now: number) {
	switch (duration) {
		case '1h':
			return now + 60 * 60_000;
		case '6h':
			return now + 6 * 60 * 60_000;
		case '24h':
			return now + 24 * 60 * 60_000;
		case '7d':
			return now + 7 * 24 * 60 * 60_000;
		default:
			return null;
	}
}

function isStoredRule(value: unknown): value is MarketNotificationRule {
	if (!value || typeof value !== 'object') return false;
	const rule = value as Partial<MarketNotificationRule>;
	return (
		typeof rule.id === 'string' &&
		typeof rule.itemId === 'string' &&
		typeof rule.slug === 'string' &&
		typeof rule.itemName === 'string' &&
		(rule.intent === 'buy' || rule.intent === 'sell') &&
		typeof rule.price === 'number' &&
		Number.isFinite(rule.price) &&
		typeof rule.createdAt === 'number' &&
		(rule.expiresAt === null || typeof rule.expiresAt === 'number') &&
		typeof rule.sessionOnly === 'boolean' &&
		(rule.notifyOnce === undefined || typeof rule.notifyOnce === 'boolean')
	);
}
