import assert from 'node:assert/strict';
import test from 'node:test';
import {
	WorldStateSchema, buildWorldSections, createResolver, timestamp,
	normalizeItemKey, rewardNames, worldCycles, safeUrl,
} from '../src/lib/worldstate.ts';

const now = Date.parse('2026-09-18T16:00:00Z');
const date = (value) => ({ $date: { $numberLong: String(value) } });
const active = { Activation: date(now - 1000), Expiry: date(now + 60000) };
const world = (extra = {}) => WorldStateSchema.parse({ Time: now / 1000, ...extra });
const section = (state, id, catalog = {}) => buildWorldSections(state, catalog).find((section) => section.id === id);

test('catalog relationships remove StoreItems, accept names, and preserve unknown items', () => {
	const resolve = createResolver({ '/Lotus/Weapons/Example': { name: 'Example Prime' }, '/Lotus/Other': { itemName: 'Other Item' } });
	assert.equal(normalizeItemKey('/Lotus/StoreItems/Weapons/Example'), '/Lotus/Weapons/Example');
	assert.equal(resolve('/Lotus/StoreItems/Weapons/Example'), 'Example Prime');
	assert.equal(resolve('example prime'), 'Example Prime');
	assert.equal(resolve('Other Item'), 'Other Item');
	assert.equal(resolve('/Lotus/Unknown/NewReward'), 'New Reward');
	assert.deepEqual(rewardNames({ countedItems: [{ ItemType: '/Lotus/StoreItems/Weapons/Example', ItemCount: 3 }], credits: 5000 }, resolve), ['3 × Example Prime', '5,000 credits']);
});

test('Mongo dates, absent sections and invalid responses are handled', () => {
	assert.equal(timestamp(date(now)), now);
	assert.equal(timestamp({ $date: now }), now);
	assert.equal(timestamp('2026-09-18T16:00:00Z'), now);
	assert.equal(timestamp({}), undefined);
	assert.equal(timestamp({ $date: { $numberLong: 'invalid' } }), undefined);
	assert.deepEqual(world({ Alerts: null }).Alerts, []);
	assert.throws(() => WorldStateSchema.parse({ error: 'upstream unavailable' }));
	assert.doesNotThrow(() => buildWorldSections(world(), {}));
});

test('fissures separate modes, exclude expired/future missions, and rank by relic tier', () => {
	const base = { ...active, Node: 'SolNode1', MissionType: 'MT_CAPTURE' };
	const state = world({ ActiveMissions: [
		{ ...base, Modifier: 'VoidT4' }, { ...base, Modifier: 'VoidT1' },
		{ ...base, Modifier: 'VoidT2', Hard: true },
		{ ...base, Modifier: 'VoidT3', Expiry: date(now) },
		{ ...base, Modifier: 'VoidT3', Activation: date(now + 10000) },
	] });
	assert.deepEqual(section(state, 'fissures').rows.map((row) => row.badge), ['Lith', 'Axi']);
	assert.equal(section(state, 'steel-fissures').rows.length, 1);
});

test('Circuit array and vendor currency types stay distinct, with missing prices omitted', () => {
	const state = world({ EndlessXpSchedule: [{ ...active, CategoryChoices: [{ Category: 'EXC_NORMAL', Choices: ['Excalibur'] }] }],
		PrimeVaultTraders: [{ ...active, Node: 'TradeHUB1', Manifest: [
			{ ItemType: '/Lotus/Example', PrimePrice: 3 }, { ItemType: '/Lotus/Relic', RegularPrice: 1 }, { ItemType: '/Lotus/Unknown' },
		], EvergreenManifest: [{ ItemType: '/Lotus/Evergreen', PrimePrice: 2 }] }],
	});
	assert.deepEqual(section(state, 'circuit').rows[0].rewards, ['Excalibur']);
	const items = section(state, 'PrimeVaultTraders').rows;
	assert.equal(items[1].detail, '3 Regal Aya');
	assert.equal(items[2].detail, '1 Aya');
	assert.equal(items[3].detail, '');
	assert.equal(items[4].title, 'Evergreen');
});

test('future Baro visits show location and arrival without presenting inventory as available', () => {
	const state = world({ VoidTraders: [{ ...active, Activation: date(now + 10000), Node: 'MercuryHUB', Manifest: [{ ItemType: '/Lotus/Example', PrimePrice: 100 }] }] });
	assert.equal(section(state, 'VoidTraders').rows.length, 1);
	assert.equal(section(state, 'VoidTraders').rows[0].activation, now + 10000);
});

test('invasion progress handles both factions and infestation, and omits completed invasions', () => {
	const base = { Node: 'SolNode1', Goal: 100, Count: -50, Faction: 'FC_CORPUS', DefenderFaction: 'FC_GRINEER' };
	const rows = section(world({ Invasions: [base, { ...base, Faction: 'FC_INFESTATION' }, { ...base, Completed: true }] }), 'invasions').rows;
	assert.deepEqual(rows.map((row) => row.progress), [25, 50]);
});

test('cycles use snapshot time and Cetus phase boundaries without requesting live data', () => {
	const state = world({ SyndicateMissions: [{ Tag: 'CetusSyndicate', Activation: date(now - 1000), Expiry: date(now + 4000000) }] });
	const cycles = worldCycles(state);
	assert.equal(cycles[0].state, 'Day');
	assert.equal(cycles[3].state, 'Fass');
	assert.ok(cycles.every((cycle) => cycle.expiry === undefined || cycle.expiry > now));
	const night = world({ SyndicateMissions: [{ Tag: 'CetusSyndicate', Expiry: date(now + 3000000) }] });
	assert.equal(worldCycles(night)[0].state, 'Night');
});

test('external news links accept only http and https', () => {
	assert.equal(safeUrl('javascript:alert(1)'), undefined);
	assert.equal(safeUrl('file:///C:/Windows'), undefined);
	assert.equal(safeUrl('https://www.warframe.com/news'), 'https://www.warframe.com/news');
});

test('1999 calendar converts day-of-year to a date and resolves rewards', () => {
	const state = world({ KnownCalendarSeasons: [{ ...active, Days: [{ day: 278, events: [{ type: 'CET_REWARD', reward: '/Lotus/StoreItems/Example' }] }] }] });
	const row = section(state, 'calendar', { '/Lotus/Example': { name: 'Calendar Reward' } }).rows[0];
	assert.equal(row.title, 'Calendar Reward');
	assert.equal(row.detail, new Date(Date.UTC(1999, 9, 5)).toLocaleDateString(undefined, { month: 'long', day: 'numeric', timeZone: 'UTC' }));
});
