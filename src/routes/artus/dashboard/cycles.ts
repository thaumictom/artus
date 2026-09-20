import type { WorldState } from 'warframe-worldstate-parser';

const MINUTE_MS = 60_000;
type CycleKey =
	| 'cetusCycle'
	| 'cambionCycle'
	| 'vallisCycle'
	| 'zarimanCycle'
	| 'duviriCycle';
type CycleDefinition = {
	key: CycleKey;
	label: string;
	states: readonly { value: string; color: string }[];
	durations: Readonly<Record<string, number>>;
};

const cycleDefinitions: readonly CycleDefinition[] = [
	{
		key: 'cetusCycle',
		label: 'Cetus',
		states: [
			{ value: 'day', color: '#E8C872' },
			{ value: 'night', color: '#385270' },
		],
		durations: { day: 100 * MINUTE_MS, night: 50 * MINUTE_MS },
	},
	{
		key: 'cambionCycle',
		label: 'Cambion Drift',
		states: [
			{ value: 'fass', color: '#E77D57' },
			{ value: 'vome', color: '#6EC1D6' },
		],
		durations: { fass: 100 * MINUTE_MS, vome: 50 * MINUTE_MS },
	},
	{
		key: 'vallisCycle',
		label: 'Orb Vallis',
		states: [
			{ value: 'warm', color: '#E09F67' },
			{ value: 'cold', color: '#79C0D0' },
		],
		durations: { warm: 400_000, cold: 1_200_000 },
	},
	{
		key: 'zarimanCycle',
		label: 'Zariman',
		states: [
			{ value: 'corpus', color: '#5B83B8' },
			{ value: 'grineer', color: '#8FA075' },
		],
		durations: { corpus: 150 * MINUTE_MS, grineer: 150 * MINUTE_MS },
	},
	{
		key: 'duviriCycle',
		label: 'Duviri',
		states: [
			{ value: 'sorrow', color: '#5A94C7' },
			{ value: 'fear', color: '#9D73C2' },
			{ value: 'joy', color: '#F0C85A' },
			{ value: 'anger', color: '#D95B66' },
			{ value: 'envy', color: '#62C085' },
		],
		durations: {
			sorrow: 120 * MINUTE_MS,
			fear: 120 * MINUTE_MS,
			joy: 120 * MINUTE_MS,
			anger: 120 * MINUTE_MS,
			envy: 120 * MINUTE_MS,
		},
	},
];

function phaseSegments(definition: CycleDefinition, stateIndex: number, progress: number) {
	const { states, durations } = definition;
	// A fixed time scale keeps the strip continuous when unequal-length phases switch.
	const windowDuration = Math.max(...Object.values(durations)) * (definition.key === 'duviriCycle' ? 2 : 1);
	let index = stateIndex;
	let left = 0.5 - (progress * durations[states[index].value]) / windowDuration;
	while (left > 0) {
		index = (index + states.length - 1) % states.length;
		left -= durations[states[index].value] / windowDuration;
	}

	const segments = [];
	while (left < 1) {
		const state = states[index];
		const width = durations[state.value] / windowDuration;
		segments.push({ ...state, left: left * 100, width: width * 100 });
		left += width;
		index = (index + 1) % states.length;
	}
	return segments;
}

function currentCycle(
	cycle: { state: string; expiry?: Date },
	definition: CycleDefinition,
	now: number,
) {
	let stateIndex = definition.states.findIndex(({ value }) => value === cycle.state);
	let expiry = cycle.expiry?.getTime();
	if (stateIndex < 0 || expiry === undefined || !Number.isFinite(expiry) || !Number.isFinite(now)) {
		return { state: cycle.state, expiry: undefined, progress: 0, nextState: cycle.state, segments: [] };
	}

	while (expiry <= now) {
		stateIndex = (stateIndex + 1) % definition.states.length;
		expiry += definition.durations[definition.states[stateIndex].value];
	}

	const state = definition.states[stateIndex].value;
	const duration = definition.durations[state];
	const progress = Math.min(1, Math.max(0, (now - (expiry - duration)) / duration));
	const nextState = definition.states[(stateIndex + 1) % definition.states.length].value;

	return { state, expiry: new Date(expiry), progress, nextState, segments: phaseSegments(definition, stateIndex, progress) };
}

export function getDashboardCycles(world: WorldState, now: number) {
	return cycleDefinitions.map((definition) => ({
		key: definition.key,
		label: definition.label,
		cycle: currentCycle(world[definition.key], definition, now),
	}));
}
