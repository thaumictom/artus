import { z } from 'zod';
import mappings from './data/worldstate-labels.json' with { type: 'json' };

type RecordData = Record<string, unknown>;
const entries = z.array(z.record(z.string(), z.unknown())).nullish().transform((value) => value ?? []);
export const WorldStateSchema = z.looseObject({
	Time: z.number().positive(),
	Events: entries, Goals: entries, Alerts: entries, Sorties: entries, LiteSorties: entries,
	SyndicateMissions: entries, ActiveMissions: entries, Invasions: entries,
	VoidTraders: entries, PrimeVaultTraders: entries, VoidStorms: entries, DailyDeals: entries,
	GlobalUpgrades: entries, Conquests: entries, Descents: entries, KnownCalendarSeasons: entries,
	WeeklyVaultBonusRewards: entries, EndlessXpSchedule: entries,
});
export type WorldState = z.infer<typeof WorldStateSchema>;
export const WorldCatalogSchema = z.record(z.string(), z.object({
	name: z.string().nullish(), itemName: z.string().nullish(), description: z.string().nullish(),
}));
export type WorldCatalog = z.infer<typeof WorldCatalogSchema>;

export type WorldRow = {
	title: string;
	detail?: string;
	badge?: string;
	rewards?: string[];
	activation?: number;
	expiry?: number;
	progress?: number;
	progressLabel?: string;
	url?: string;
};
export type WorldSection = {
	id: string;
	title: string;
	icon: string;
	rows: WorldRow[];
	empty?: string;
	note?: string;
	collapsed?: boolean;
};
export type WorldCycle = { title: string; state: string; expiry?: number };

export const object = (value: unknown): RecordData =>
	value !== null && typeof value === 'object' && !Array.isArray(value) ? value as RecordData : {};
const array = (value: unknown): unknown[] => Array.isArray(value) ? value : [];
const records = (value: unknown) => array(value).map(object);
const string = (value: unknown) => typeof value === 'string' ? value : '';
const number = (value: unknown) => typeof value === 'number' && Number.isFinite(value) ? value : undefined;
const format = (value: number) => value.toLocaleString();
const join = (...parts: (string | undefined)[]) => parts.filter(Boolean).join(' · ');

export function timestamp(value: unknown): number | undefined {
	const date = object(value).$date ?? value;
	const raw = object(date).$numberLong ?? date;
	if (typeof raw !== 'number' && typeof raw !== 'string') return undefined;
	const result = typeof raw === 'string' && !/^\d+$/.test(raw) ? Date.parse(raw) : Number(raw);
	return Number.isFinite(result) && result > 0 ? result : undefined;
}

export const normalizeItemKey = (key: string) => key.replaceAll('/StoreItems', '');
const cleanText = (value: string) => value.replace(/<[^>]*>/g, '').replace(/\|[^|]*\|/g, '…');
export function readable(value: unknown): string {
	const raw = string(value);
	return cleanText(raw.split('/').at(-1)?.replace(/^(MT_|FC_|DT_|CST_|CET_)/, '')
		.replace(/([a-z0-9])([A-Z])/g, '$1 $2').replaceAll('_', ' ') ?? '');
}

type Labels = Record<string, { value: string; desc?: string }>;
const languages: Labels = mappings.languages;
const nodes: Record<string, { value: string; type?: string; enemy?: string }> = mappings.nodes;
const missions: Labels = mappings.missions;
const factions: Labels = mappings.factions;
const fissures: Record<string, { value: string; num: number }> = mappings.fissures;
const syndicates: Record<string, { name: string }> = mappings.syndicates;
const bosses: Record<string, { name: string }> = mappings.sortie.bosses;
const modifiers: Record<string, string> = mappings.sortie.modifierTypes;

export function createResolver(catalog: WorldCatalog) {
	const byKey = new Map(Object.entries(catalog).map(([key, item]) => [normalizeItemKey(key).toLowerCase(), item]));
	const byName = new Map(Object.values(catalog).flatMap((item) =>
		[item.name, item.itemName].filter((name): name is string => Boolean(name)).map((name) => [name.toLowerCase(), item] as const)));
	return (value: unknown): string => {
		const raw = string(value);
		if (!raw) return 'Unknown item';
		const key = normalizeItemKey(raw).toLowerCase();
		const item = byKey.get(key) ?? byName.get(raw.toLowerCase());
		return cleanText(item?.name || item?.itemName || languages[raw.toLowerCase()]?.value || languages[key]?.value || readable(raw));
	};
}
const description = (value: unknown) => cleanText(languages[string(value).toLowerCase()]?.desc ?? '');
const nodeName = (value: unknown) => nodes[string(value)]?.value ?? readable(value);
const missionName = (value: unknown) => missions[string(value)]?.value ?? readable(value);
const factionName = (value: unknown) => factions[string(value)]?.value ?? readable(value);
const calendarDate = (value: unknown) => {
	const day = number(value);
	return day !== undefined && day >= 1 && day <= 365
		? new Date(Date.UTC(1999, 0, day)).toLocaleDateString(undefined, { month: 'long', day: 'numeric', timeZone: 'UTC' })
		: 'Unknown date';
};
const timing = (row: RecordData) => ({ activation: timestamp(row.Activation), expiry: timestamp(row.Expiry) });
export function isCurrent(row: RecordData, now: number) {
	const { activation, expiry } = timing(row);
	return (activation === undefined || activation <= now) && (expiry === undefined || expiry > now);
}
export function safeUrl(value: unknown): string | undefined {
	try {
		const url = new URL(string(value));
		return ['https:', 'http:'].includes(url.protocol) ? url.href : undefined;
	} catch { return undefined; }
}

export function rewardNames(value: unknown, resolve: (value: unknown) => string): string[] {
	const reward = object(value);
	const result = array(reward.items).map(resolve);
	for (const item of records(reward.countedItems)) {
		result.push(`${format(number(item.ItemCount) ?? 1)} × ${resolve(item.ItemType)}`);
	}
	const credits = number(reward.credits);
	if (credits) result.push(`${format(credits)} credits`);
	return result;
}

/** Environment anchors follow WFCD's worldstate-parser cycle models.
 * Calculate at snapshot time; an expired card asks for Reload instead of inventing a new snapshot.
 */
export function worldCycles(world: WorldState): WorldCycle[] {
	const now = world.Time * 1000;
	const cycle = (title: string, anchor: number, phases: [string, number][]): WorldCycle => {
		const period = phases.reduce((total, [, duration]) => total + duration, 0);
		let elapsed = ((now - anchor) % period + period) % period;
		for (const [state, duration] of phases) {
			if (elapsed < duration) return { title, state, expiry: now + duration - elapsed };
			elapsed -= duration;
		}
		return { title, state: 'Unavailable' };
	};
	const cetus = world.SyndicateMissions.find((row) => row.Tag === 'CetusSyndicate' && isCurrent(row, now));
	const cetusEnd = timestamp(cetus?.Expiry);
	const plains = cetusEnd ? cycle('Plains of Eidolon', Math.floor(cetusEnd / 60000) * 60000, [['Day', 6000000], ['Night', 3000000]])
		: { title: 'Plains of Eidolon', state: 'Unavailable' };
	const zariman = world.SyndicateMissions.find((row) => row.Tag === 'ZarimanSyndicate' && isCurrent(row, now));
	const zarimanEnd = timestamp(zariman?.Expiry);
	const zarimanPhase = zarimanEnd === undefined ? undefined : ((zarimanEnd - 5000 - 1655182800000) % 18000000 + 18000000) % 18000000;
	return [
		plains,
		cycle('Earth', 0, [['Day', 14400000], ['Night', 14400000]]),
		cycle('Orb Vallis', Date.parse('2026-02-04T19:46:48Z'), [['Warm', 400000], ['Cold', 1200000]]),
		{ ...plains, title: 'Cambion Drift', state: plains.state === 'Day' ? 'Fass' : plains.state === 'Night' ? 'Vome' : 'Unavailable' },
		cycle('Duviri', 52000, ['Sorrow', 'Fear', 'Joy', 'Anger', 'Envy'].map((state) => [state, 7200000])),
		{ title: 'Zariman', state: zarimanPhase === undefined ? 'Unavailable' : zarimanPhase < 9000000 ? 'Corpus' : 'Grineer', expiry: zarimanEnd },
	];
}

export function buildWorldSections(world: WorldState, catalog: WorldCatalog): WorldSection[] {
	const now = world.Time * 1000;
	const resolve = createResolver(catalog);
	const rewards = (value: unknown) => rewardNames(value, resolve);
	const current = (rows: RecordData[]) => rows.filter((row) => isCurrent(row, now));
	const sections: WorldSection[] = [];
	const add = (id: string, title: string, icon: string, rows: WorldRow[], extra: Partial<WorldSection> = {}) => {
		sections.push({ id, title, icon: `material-symbols:${icon}`, rows, ...extra });
	};
	for (const [key, title, icon] of [['Sorties', 'Sortie', 'target'], ['LiteSorties', 'Archon Hunt', 'swords']] as const) {
		const rows = current(world[key]).flatMap((sortie) => {
			const boss = bosses[string(sortie.Boss)]?.name ?? readable(sortie.Boss);
			return records(sortie.Variants ?? sortie.Missions).map((mission, index) => ({
				title: `${index + 1}. ${missionName(mission.missionType)} — ${nodeName(mission.node)}`,
				detail: join(boss, modifiers[string(mission.modifierType)] ?? readable(mission.modifierType)),
				...timing(sortie),
			}));
		});
		add(key, title, icon, rows, { empty: 'No active missions in this snapshot.' });
	}
	add('alerts', 'Alerts', 'notification-important-outline', current(world.Alerts).map((alert) => {
		const mission = object(alert.MissionInfo);
		return { title: nodeName(mission.location), detail: join(missionName(mission.missionType), factionName(mission.faction),
			mission.minEnemyLevel !== undefined ? `Level ${mission.minEnemyLevel}–${mission.maxEnemyLevel}` : ''),
			rewards: rewards(mission.missionReward), ...timing(alert) };
	}), { empty: 'No active alerts.' });
	add('events', 'Events & boosters', 'celebration-outline', [
		...current(world.Goals).map((goal) => ({ title: resolve(goal.Desc ?? goal.Tag), detail: join(resolve(goal.ToolTip ?? goal.Desc), nodeName(goal.Node)),
			rewards: [...rewards(goal.Reward), ...records(goal.InterimRewards).flatMap(rewards)], ...timing(goal),
			progress: number(goal.HealthPct) === undefined ? undefined : Math.max(0, Math.min(100, Number(goal.HealthPct) * 100)), progressLabel: 'Community progress' })),
		...current(world.GlobalUpgrades).map((upgrade) => ({ title: resolve(upgrade.UpgradeType),
			detail: `${number(upgrade.Value) ?? ''} ${readable(upgrade.OperationType)}`, ...timing(upgrade) })),
	], { empty: 'No active events or global boosters.' });

	const fissureRows = (rows: RecordData[]): WorldRow[] => current(rows).sort((a, b) =>
		(fissures[string(a.Modifier ?? a.ActiveMissionTier)]?.num ?? 99) - (fissures[string(b.Modifier ?? b.ActiveMissionTier)]?.num ?? 99)
		|| (timestamp(a.Expiry) ?? 0) - (timestamp(b.Expiry) ?? 0)).map((fissure) => ({
			title: nodeName(fissure.Node), badge: fissures[string(fissure.Modifier ?? fissure.ActiveMissionTier)]?.value ?? readable(fissure.Modifier ?? fissure.ActiveMissionTier),
			detail: join(missionName(fissure.MissionType) || nodes[string(fissure.Node)]?.type, nodes[string(fissure.Node)]?.enemy), ...timing(fissure),
		}));
	add('fissures', 'Void Fissures', 'flare', fissureRows(world.ActiveMissions.filter((row) => !row.Hard)), { empty: 'No normal fissures.' });
	add('steel-fissures', 'Steel Path Fissures', 'swords', fissureRows(world.ActiveMissions.filter((row) => row.Hard)), { empty: 'No Steel Path fissures.' });
	add('storms', 'Railjack Void Storms', 'rocket-launch-outline', fissureRows(world.VoidStorms), { empty: 'No active Void Storms.' });

	add('invasions', 'Invasions', 'compare-arrows', world.Invasions.filter((row) => !row.Completed && isCurrent(row, now)).map((invasion) => {
		const attacker = invasion.Faction ?? object(invasion.DefenderMissionInfo).faction;
		const defender = invasion.DefenderFaction ?? object(invasion.AttackerMissionInfo).faction;
		const goal = number(invasion.Goal);
		const count = number(invasion.Count);
		const percent = goal && count !== undefined ? (1 + count / goal) * (attacker === 'FC_INFESTATION' ? 100 : 50) : undefined;
		return { title: nodeName(invasion.Node), detail: `${factionName(attacker)} vs ${factionName(defender)}`,
			rewards: [
				...rewards(invasion.AttackerReward).map((reward) => `${factionName(attacker)}: ${reward}`),
				...rewards(invasion.DefenderReward).map((reward) => `${factionName(defender)}: ${reward}`),
			], progress: percent === undefined ? undefined : Math.max(0, Math.min(100, percent)),
			progressLabel: attacker === 'FC_INFESTATION' ? 'Infested strength remaining' : `${factionName(attacker)} control` };
	}), { empty: 'No active invasions.' });

	for (const [key, title] of [['VoidTraders', "Baro Ki’Teer"], ['PrimeVaultTraders', 'Prime Resurgence']] as const) {
		const traders = world[key].filter((trader) => !timestamp(trader.Expiry) || timestamp(trader.Expiry)! > now);
		const rows: WorldRow[] = traders.flatMap((trader) => {
			const summary: WorldRow = { title: nodeName(trader.Node), detail: key === 'VoidTraders' ? 'Void Trader visit' : 'Varzia’s offerings', ...timing(trader) };
			if ((summary.activation ?? 0) > now) return [summary];
			return [summary, ...[...records(trader.Manifest), ...records(trader.EvergreenManifest)].map((item) => ({ title: resolve(item.ItemType),
				detail: join(number(item.PrimePrice) === undefined ? '' : `${format(Number(item.PrimePrice))} ${key === 'VoidTraders' ? 'ducats' : 'Regal Aya'}`,
					number(item.RegularPrice) === undefined ? '' : `${format(Number(item.RegularPrice))} ${key === 'VoidTraders' ? 'credits' : 'Aya'}`,
					item.Limit ? `Limit ${item.Limit}` : ''), ...timing(trader) }))];
		});
		add(key, title, 'storefront-outline', rows, { collapsed: key === 'PrimeVaultTraders', empty: 'No visit information in this snapshot.' });
	}
	add('darvo', 'Darvo’s Deal', 'sell-outline', current(world.DailyDeals).map((deal) => ({
		title: resolve(deal.StoreItem), badge: `${deal.Discount ?? 0}% off`,
		detail: `${deal.SalePrice ?? '?'} platinum (was ${deal.OriginalPrice ?? '?'}) · ${Math.max(0, (number(deal.AmountTotal) ?? 0) - (number(deal.AmountSold) ?? 0))}/${deal.AmountTotal ?? '?'} in stock`, ...timing(deal),
	})), { empty: 'No active deal.' });

	const season = object(world.SeasonInfo);
	add('nightwave', 'Nightwave', 'radio-outline', current(records(season.ActiveChallenges)).map((challenge) => ({
		title: resolve(challenge.Challenge), detail: description(challenge.Challenge),
		badge: challenge.Daily ? 'Daily · 1,000' : /weeklyhard/i.test(string(challenge.Challenge)) ? 'Elite weekly · 7,000' : 'Weekly · 4,500',
		...timing(challenge),
	})), { empty: 'No active Nightwave challenges.' });
	add('circuit', 'The Circuit', 'all-inclusive', current(world.EndlessXpSchedule).flatMap((circuit) => records(circuit.CategoryChoices).map((choice) => ({
		title: choice.Category === 'EXC_HARD' ? 'Steel Path · Incarnon adapters' : 'Normal · Warframes',
		rewards: array(choice.Choices).map(resolve), ...timing(circuit),
	}))), { empty: 'No current Circuit rotation.' });

	for (const conquest of current(world.Conquests)) {
		const hex = conquest.Type === 'CT_HEX';
		add(`archimedea-${string(conquest.Type)}`, hex ? 'Temporal Archimedea' : 'Deep Archimedea', 'science-outline', [
			...records(conquest.Missions).map((mission, index) => ({ title: `${index + 1}. ${missionName(mission.missionType)}`,
				detail: factionName(mission.faction), ...timing(conquest),
				rewards: records(mission.difficulties).map((difficulty) => `${difficulty.type === 'CD_HARD' ? 'Elite' : 'Normal'}: ${[difficulty.deviation, ...array(difficulty.risks)].map(resolve).join(' · ')}`) })),
			...array(conquest.Variables).map((variable) => ({ title: resolve(variable), detail: description(variable), badge: 'Personal modifier' })),
		], { collapsed: true });
	}
	for (const descent of current(world.Descents)) {
		add('descendia', 'Descendia', 'stairs', records(descent.Challenges).map((challenge) => ({
			title: `${challenge.Index}. ${resolve(challenge.Challenge)}`, detail: readable(challenge.Type),
			rewards: array(challenge.Auras).map(resolve), ...timing(descent),
		})), { collapsed: true });
	}

	const bountyTags = ['CetusSyndicate', 'SolarisSyndicate', 'EntratiSyndicate', 'ZarimanSyndicate', 'EntratiLabSyndicate', 'HexSyndicate'];
	const seenBounties = new Set<string>();
	for (const bounty of current(world.SyndicateMissions).filter((row) => bountyTags.includes(string(row.Tag)))) {
		const tag = string(bounty.Tag);
		if (seenBounties.has(tag)) continue;
		seenBounties.add(tag);
		const jobs = records(bounty.Jobs);
		add(`bounty-${tag}`, `${syndicates[tag]?.name ?? readable(tag)} bounties`, 'assignment-outline', jobs.length ? jobs.map((job, index) => ({
			title: job.jobType ? resolve(job.jobType) : job.isVault ? `Isolation Vault ${index + 1}` : `Bounty ${index + 1}`,
			detail: join(`Level ${job.minEnemyLevel ?? '?'}–${job.maxEnemyLevel ?? '?'}`, job.masteryReq ? `MR ${job.masteryReq}` : '',
				array(job.xpAmounts).length ? `${format(array(job.xpAmounts).reduce<number>((sum, xp) => sum + (number(xp) ?? 0), 0))} ${tag === 'EntratiSyndicate' ? 'Mother tokens' : 'standing'}` : ''),
			badge: /Table([A-Z])Rewards/.exec(string(job.rewards))?.[1] ? `Rotation ${/Table([A-Z])Rewards/.exec(string(job.rewards))![1]}` : undefined,
			...timing(bounty),
		})) : [{ title: 'Current bounty rotation', detail: 'Mission objectives and rewards are not included in the official feed.', ...timing(bounty) }], { collapsed: true });
	}
	add('syndicates', 'Syndicate missions', 'groups-outline', current(world.SyndicateMissions).flatMap((syndicate) => array(syndicate.Nodes).map((node) => ({
		title: nodeName(node), detail: syndicates[string(syndicate.Tag)]?.name ?? readable(syndicate.Tag), ...timing(syndicate),
	}))), { collapsed: true, empty: 'No syndicate missions.' });

	add('calendar', '1999 Calendar', 'calendar-month-outline', current(world.KnownCalendarSeasons).flatMap((calendar) => records(calendar.Days).flatMap((day) =>
		records(day.events).map((event) => ({ title: resolve(event.challenge ?? event.reward ?? event.upgrade ?? event.dialogueName ?? 'Calendar event'),
			detail: join(description(event.challenge ?? event.upgrade), calendarDate(day.day)),
			badge: readable(event.type), ...timing(calendar),
		})))), { collapsed: true, empty: 'No current calendar events.' });
	add('vault', 'Weekly vault rotations', 'redeem-outline', world.WeeklyVaultBonusRewards.flatMap((bonus) => records(bonus.Rewards).map((reward) => ({
		title: `${reward.ItemCount ?? 1} × ${resolve(reward.Reward)}`, detail: resolve(bonus.BonusRegion), badge: `${format(number(reward.PointThreshold) ?? 0)} points`,
	}))), { collapsed: true, empty: 'No weekly vault rewards.', note: 'All reward rotations supplied by the feed are shown.' });
	add('news', 'News', 'newspaper', current(world.Events).filter((event) => !event.MobileOnly).map((event) => {
		const message = records(event.Messages).find((message) => message.LanguageCode === 'en');
		return { title: resolve(message?.Message ?? event.Message), url: safeUrl(event.Prop), activation: timestamp(event.Date),
			badge: event.Priority ? 'Featured' : event.Community ? 'Community' : undefined };
	}).filter((event) => event.title !== 'Unknown item').reverse(), { collapsed: true, empty: 'No news in this snapshot.' });
	return sections;
}

export function duration(milliseconds: number): string {
	const seconds = Math.max(0, Math.floor(milliseconds / 1000));
	if (seconds >= 86400) return `${Math.floor(seconds / 86400)}d ${Math.floor(seconds % 86400 / 3600)}h`;
	if (seconds >= 3600) return `${Math.floor(seconds / 3600)}h ${Math.floor(seconds % 3600 / 60)}m`;
	if (seconds >= 60) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
	return `${seconds}s`;
}

export function fetchedAgo(fetchedAt: number, now: number): string {
	return now - fetchedAt < 3000 ? 'just now' : `${duration(now - fetchedAt)} ago`;
}
