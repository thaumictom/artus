import { LazyStore } from '@tauri-apps/plugin-store';
import type { DayEvent, WorldState } from 'warframe-worldstate-parser';

const store = new LazyStore('calendar-completions.json');
let pending: Promise<unknown> = Promise.resolve();
export const calendarCompletions = $state({ completedIds: [] as string[], loaded: false });
const completedIdSet = $derived(new Set(calendarCompletions.completedIds));

export function hasCalendarTodo(events: DayEvent[]): boolean {
	return events.some((event) => event.type.toLowerCase() === 'to do');
}

export function getCalendarDays(calendar: WorldState['calendar']) {
	const days = new Map<string, { id: string; date: Date; events: DayEvent[] }>();
	if (!calendar) return [];
	for (const day of calendar.days ?? []) {
		const date = new Date(day.date);
		if (!Number.isFinite(date.getTime()) || day.events.length === 0) continue;
		const dateKey = date.toISOString().slice(0, 10);
		// Completion belongs to this season and loop, not the same date in a later loop.
		const id = `${calendar.yearIteration}:${calendar.season}:${dateKey}`;
		const existing = days.get(id);
		if (existing) existing.events.push(...day.events);
		else days.set(id, { id, date: new Date(`${dateKey}T00:00:00Z`), events: [...day.events] });
	}
	return [...days.values()].sort((a, b) => a.date.getTime() - b.date.getTime());
}

export function isCalendarDayCompleted(id: string): boolean {
	return completedIdSet.has(id);
}

function withStore<T>(action: () => Promise<T>): Promise<T> {
	const result = pending.then(action);
	pending = result.catch(() => {});
	return result;
}

export function initializeCalendarCompletions(): Promise<void> {
	return withStore(async () => {
		if (calendarCompletions.loaded) return;
		calendarCompletions.completedIds = await store.get<string[]>('completedIds') ?? [];
		calendarCompletions.loaded = true;
	});
}

export function setCalendarDaysCompleted(ids: string[], completed: boolean): Promise<void> {
	return withStore(async () => {
		const saved = await store.get<string[]>('completedIds') ?? [];
		const selected = new Set(ids);
		const updated = completed
			? [...new Set([...saved, ...ids])]
			: saved.filter((id) => !selected.has(id));
		await store.set('completedIds', updated);
		await store.save();
		calendarCompletions.completedIds = updated;
		calendarCompletions.loaded = true;
	});
}
