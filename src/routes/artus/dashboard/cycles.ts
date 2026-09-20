import type { WorldState } from 'warframe-worldstate-parser';

const MINUTE_MS = 60_000;
type CycleKey =
	| 'earthCycle'
	| 'cetusCycle'
	| 'cambionCycle'
	| 'vallisCycle'
	| 'zarimanCycle'
	| 'duviriCycle';
type CycleDefinition = {
	key: CycleKey;
	label: string;
	states: readonly string[];
	durations: Readonly<Record<string, number>>;
};

const cycleDefinitions: readonly CycleDefinition[] = [
	{
		key: 'earthCycle',
		label: 'Earth',
		states: ['day', 'night'],
		durations: { day: 240 * MINUTE_MS, night: 240 * MINUTE_MS },
	},
	{
		key: 'cetusCycle',
		label: 'Cetus',
		states: ['day', 'night'],
		durations: { day: 100 * MINUTE_MS, night: 50 * MINUTE_MS },
	},
	{
		key: 'cambionCycle',
		label: 'Cambion Drift',
		states: ['fass', 'vome'],
		durations: { fass: 100 * MINUTE_MS, vome: 50 * MINUTE_MS },
	},
	{
		key: 'vallisCycle',
		label: 'Orb Vallis',
		states: ['warm', 'cold'],
		durations: { warm: 400_000, cold: 1_200_000 },
	},
	{
		key: 'zarimanCycle',
		label: 'Zariman',
		states: ['corpus', 'grineer'],
		durations: { corpus: 150 * MINUTE_MS, grineer: 150 * MINUTE_MS },
	},
	{
		key: 'duviriCycle',
		label: 'Duviri',
		states: ['sorrow', 'fear', 'joy', 'anger', 'envy'],
		durations: {
			sorrow: 120 * MINUTE_MS,
			fear: 120 * MINUTE_MS,
			joy: 120 * MINUTE_MS,
			anger: 120 * MINUTE_MS,
			envy: 120 * MINUTE_MS,
		},
	},
];

function currentCycle(
	cycle: { state: string; expiry?: Date },
	definition: CycleDefinition,
	now: number,
) {
	let stateIndex = definition.states.indexOf(cycle.state);
	let expiry = cycle.expiry?.getTime();
	if (stateIndex < 0 || expiry === undefined) {
		return { ...cycle, progress: 0, nextState: cycle.state };
	}

	while (expiry <= now) {
		stateIndex = (stateIndex + 1) % definition.states.length;
		expiry += definition.durations[definition.states[stateIndex]];
	}

	const state = definition.states[stateIndex];
	const duration = definition.durations[state];
	const progress = Math.min(1, Math.max(0, (now - (expiry - duration)) / duration));
	const nextState = definition.states[(stateIndex + 1) % definition.states.length];

	return { state, expiry: new Date(expiry), progress, nextState };
}

export function getDashboardCycles(world: WorldState, now: number) {
	return cycleDefinitions.map((definition) => ({
		key: definition.key,
		label: definition.label,
		cycle: currentCycle(world[definition.key], definition, now),
	}));
}
