import {
	isPermissionGranted,
	onAction,
	requestPermission,
	sendNotification,
} from '@tauri-apps/plugin-notification';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { LazyStore } from '@tauri-apps/plugin-store';
import type { WorldState } from 'warframe-worldstate-parser';
import { config, type FissureNotificationCategory } from '$lib/settings.svelte';
import { openMarketNotificationTarget } from '$lib/market-navigation.svelte';

export type NotificationSource =
	| 'fissure'
	| 'alert'
	| 'invasion'
	| 'dailyDeal'
	| 'baro'
	| 'market';

export type ArtusNotification = {
	id: string;
	source: NotificationSource;
	sourceId: string;
	title: string;
	body: string;
	era?: string;
	market?: {
		slug: string;
		orderType: 'buy' | 'sell';
		since?: number;
	};
	createdAt: number;
	read: boolean;
};

const HISTORY_KEY = 'entries';
const MAX_HISTORY_LENGTH = 100;
const historyStore = new LazyStore('notifications.json');
let historySaveQueue = Promise.resolve();
let historyLoaded: Promise<void> | null = null;
let notificationActionListener: Promise<void> | null = null;
let notificationSound: HTMLAudioElement | null = null;

export const notificationCenter = $state({
	entries: [] as ArtusNotification[],
	loaded: false,
});

export function hasActiveNotificationRules() {
	const rules = config.notification_rules;
	return Boolean(
		rules?.fissures?.enabled ||
			rules?.alerts ||
			rules?.invasions ||
			rules?.dailyDeals ||
			rules?.baro,
	);
}

export function initializeNotificationCenter() {
	if (historyLoaded) return historyLoaded;
	historyLoaded = (async () => {
		try {
			await initializeNotificationActionListener();
			const saved = await historyStore.get<ArtusNotification[]>(HISTORY_KEY);
			if (Array.isArray(saved)) {
				notificationCenter.entries = saved
					.filter(isStoredNotification)
					.map(normalizeStoredNotification)
					.sort((a, b) => b.createdAt - a.createdAt)
					.slice(0, MAX_HISTORY_LENGTH);
			}
		} catch (error) {
			console.error('Could not load notification history:', error);
		} finally {
			notificationCenter.loaded = true;
		}
	})();
	return historyLoaded;
}

export async function requestDesktopNotificationPermission() {
	try {
		if (await isPermissionGranted()) return true;
		return (await requestPermission()) === 'granted';
	} catch (error) {
		console.error('Could not request desktop notification permission:', error);
		return false;
	}
}

export async function markAllNotificationsRead() {
	if (!notificationCenter.entries.some((entry) => !entry.read)) return;
	for (const entry of notificationCenter.entries) entry.read = true;
	await persistHistory();
}

export async function clearNotificationHistory() {
	notificationCenter.entries = [];
	await persistHistory();
}

export async function publishNotification(
	notification: Omit<ArtusNotification, 'id' | 'createdAt' | 'read'>,
) {
	await initializeNotificationCenter();
	await addNotification(notification);
}

export async function processWorldStateNotifications(
	world: WorldState,
	previousWorld: WorldState | null,
) {
	await initializeNotificationCenter();
	// The first successful response establishes a baseline. Existing world-state
	// entries should not flood a fresh installation as "new" notifications.
	if (!previousWorld) return;

	const rules = config.notification_rules;
	const candidates: Omit<ArtusNotification, 'id' | 'createdAt' | 'read'>[] = [];
	const now = world.timestamp.getTime();

	if (rules.fissures.enabled) {
		const previousIds = new Set(previousWorld.fissures.map(fissureKey));
		for (const fissure of world.fissures) {
			const sourceId = fissureKey(fissure);
			if (
				previousIds.has(sourceId) ||
				!isActiveAt(fissure, now) ||
				!matchesFissureRule(fissure)
			) {
				continue;
			}
			candidates.push({
				source: 'fissure',
				sourceId,
				title: `${fissure.missionType} fissure`,
				era: fissure.tier,
				body: [
					fissure.node,
					fissure.isStorm ? 'Void Storm' : fissure.isHard ? 'Steel Path' : 'Normal',
				].join(' · '),
			});
		}
	}

	if (rules.alerts) {
		const previousIds = new Set(previousWorld.alerts.map(alertKey));
		for (const alert of world.alerts) {
			const sourceId = alertKey(alert);
			if (previousIds.has(sourceId) || !isActiveAt(alert, now)) continue;
			candidates.push({
				source: 'alert',
				sourceId,
				title: `New alert: ${alert.mission.node}`,
				body: [alert.mission.type, alert.description].filter(Boolean).join(' · '),
			});
		}
	}

	if (rules.invasions) {
		const previousIds = new Set(previousWorld.invasions.map(invasionKey));
		for (const invasion of world.invasions) {
			const sourceId = invasionKey(invasion);
			if (previousIds.has(sourceId) || invasion.completed) continue;
			if (rules.invasionExcludeCommonRewards && hasOnlyExcludedInvasionRewards(invasion)) continue;
			candidates.push({
				source: 'invasion',
				sourceId,
				title: `New invasion: ${invasion.node}`,
				body: invasion.desc,
			});
		}
	}

	if (rules.dailyDeals) {
		const previousIds = new Set(previousWorld.dailyDeals.map(dailyDealKey));
		for (const deal of world.dailyDeals) {
			const sourceId = dailyDealKey(deal);
			if (previousIds.has(sourceId) || !isActiveAt(deal, now)) continue;
			candidates.push({
				source: 'dailyDeal',
				sourceId,
				title: `New daily deal: ${deal.item}`,
				body: `${deal.discount}% off · ${deal.salePrice.toLocaleString()} platinum`,
			});
		}
	}

	if (rules.baro) {
		const trader = world.voidTrader;
		const previousTrader = previousWorld.voidTrader;
		const wasActive = isActiveAt(previousTrader, previousWorld.timestamp.getTime());
		if (isActiveAt(trader, now) && (!wasActive || trader.psId !== previousTrader.psId)) {
			candidates.push({
				source: 'baro',
				sourceId: trader.psId || dateKey(trader.activation),
				title: "Baro Ki'Teer has arrived",
				body: trader.location || 'Check the Star Chart for his relay.',
			});
		}
	}

	for (const candidate of candidates) await addNotification(candidate);
}

function matchesFissureRule(fissure: WorldState['fissures'][number]) {
	const rule = config.notification_rules.fissures;
	const category: FissureNotificationCategory = fissure.isStorm
		? 'voidStorm'
		: fissure.isHard
			? 'steelPath'
			: 'normal';
	return (
		(rule.eras.length === 0 || rule.eras.includes(fissure.tier)) &&
		(rule.missionTypes.length === 0 || rule.missionTypes.includes(fissure.missionType)) &&
		(rule.categories.length === 0 || rule.categories.includes(category))
	);
}

async function addNotification(candidate: Omit<ArtusNotification, 'id' | 'createdAt' | 'read'>) {
	// A repeated API snapshot or an overlapping manual refresh must never add the
	// same world-state entry twice.
	if (
		notificationCenter.entries.some(
			(entry) => entry.source === candidate.source && entry.sourceId === candidate.sourceId,
		)
	) {
		return;
	}

	const createdAt = Date.now();
	const entry: ArtusNotification = {
		...candidate,
		id: `${candidate.source}:${candidate.sourceId}:${createdAt}`,
		createdAt,
		read: false,
	};
	notificationCenter.entries = [entry, ...notificationCenter.entries].slice(
		0,
		MAX_HISTORY_LENGTH,
	);
	await persistHistory();
	playNotificationSound();

	if (config.desktop_notifications_enabled && (await requestDesktopNotificationPermission())) {
		sendNotification({
			title: entry.era ? `${entry.era} ${entry.title}` : entry.title,
			body: entry.body,
			autoCancel: true,
			extra: entry.market
				? {
						artusAction: 'open-market-item',
						marketSlug: entry.market.slug,
						marketOrderType: entry.market.orderType,
						marketSince: entry.market.since ?? entry.createdAt,
					}
				: undefined,
		});
	}
}

function initializeNotificationActionListener() {
	if (notificationActionListener) return notificationActionListener;
	notificationActionListener = onAction((notification) => {
		const extra = notification.extra;
		if (extra?.artusAction !== 'open-market-item') return;
		const slug = extra.marketSlug;
		const orderType = extra.marketOrderType;
		const since = extra.marketSince;
		if (
			typeof slug !== 'string' ||
			(orderType !== 'buy' && orderType !== 'sell') ||
			typeof since !== 'number'
		) {
			return;
		}

		openMarketNotificationTarget(slug, since, orderType);
		const appWindow = getCurrentWindow();
		void appWindow.show();
		void appWindow.unminimize();
		void appWindow.setFocus();
	})
		.then(() => undefined)
		.catch((error) => console.error('Could not listen for notification actions:', error));
	return notificationActionListener;
}

function playNotificationSound() {
	if (!config.notification_sound || typeof Audio === 'undefined') return;
	notificationSound ??= new Audio('/sound/beep.wav');
	notificationSound.preload = 'auto';
	notificationSound.currentTime = 0;
	void notificationSound
		.play()
		.catch((error) => console.error('Could not play notification sound:', error));
}

function persistHistory() {
	historySaveQueue = historySaveQueue
		.catch(() => undefined)
		.then(async () => {
			await historyStore.set(HISTORY_KEY, $state.snapshot(notificationCenter.entries));
			await historyStore.save();
		})
		.catch((error) => console.error('Could not save notification history:', error));
	return historySaveQueue;
}

function isActiveAt(item: { activation?: Date; expiry?: Date } | undefined, now: number) {
	return Boolean(
		item &&
			(!validDate(item.activation) || item.activation.getTime() <= now) &&
			(!validDate(item.expiry) || item.expiry.getTime() > now),
	);
}

function validDate(value?: Date): value is Date {
	return value instanceof Date && Number.isFinite(value.getTime());
}

function dateKey(value?: Date) {
	return validDate(value) ? value.toISOString() : 'unknown';
}

function fissureKey(fissure: WorldState['fissures'][number]) {
	return fissure.id || `${fissure.nodeKey}:${fissure.tier}:${dateKey(fissure.activation)}`;
}

function alertKey(alert: WorldState['alerts'][number]) {
	return alert.id || `${alert.mission.node}:${dateKey(alert.activation)}`;
}

function invasionKey(invasion: WorldState['invasions'][number]) {
	return invasion.id || `${invasion.nodeKey}:${dateKey(invasion.activation)}`;
}

const excludedInvasionRewards = new Set([
	'fieldron',
	'detonite injector',
	'mutagen mass',
	'mutalist alad v nav coordinate',
]);

function hasOnlyExcludedInvasionRewards(invasion: WorldState['invasions'][number]) {
	// Keep invasions that offer any other item on either side.
	const rewards = [invasion.attacker.reward, invasion.defender.reward].flatMap((reward) =>
		reward?.countedItems?.length
			? reward.countedItems.map((item) => item.type)
			: (reward?.items ?? []),
	);
	return (
		rewards.length > 0 &&
		rewards.every((item) =>
			excludedInvasionRewards.has(item.trim().replace(/\s+/g, ' ').toLowerCase()),
		)
	);
}

function dailyDealKey(deal: WorldState['dailyDeals'][number]) {
	return deal.id || `${deal.uniqueName}:${dateKey(deal.expiry)}`;
}

function isStoredNotification(value: unknown): value is ArtusNotification {
	if (!value || typeof value !== 'object') return false;
	const entry = value as Partial<ArtusNotification>;
	return (
		typeof entry.id === 'string' &&
		typeof entry.source === 'string' &&
		typeof entry.sourceId === 'string' &&
		typeof entry.title === 'string' &&
		typeof entry.body === 'string' &&
		(entry.era === undefined || typeof entry.era === 'string') &&
		(entry.market === undefined ||
			(entry.market !== null &&
				typeof entry.market === 'object' &&
				typeof entry.market.slug === 'string' &&
				(entry.market.orderType === 'buy' || entry.market.orderType === 'sell') &&
				(entry.market.since === undefined || typeof entry.market.since === 'number'))) &&
		typeof entry.createdAt === 'number' &&
		typeof entry.read === 'boolean'
	);
}

function normalizeStoredNotification(entry: ArtusNotification): ArtusNotification {
	if (entry.source !== 'fissure' || entry.era) return entry;
	const match = /^(Lith|Meso|Neo|Axi|Requiem|Omnia) (.+)$/i.exec(entry.title);
	if (!match) return entry;
	const era = match[1][0].toUpperCase() + match[1].slice(1).toLowerCase();
	return { ...entry, era, title: match[2] };
}
