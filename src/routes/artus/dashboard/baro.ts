import type { WorldState } from 'warframe-worldstate-parser';
import { isCurrent, validDate } from './views/view-types';

export function isBaroActive(trader: WorldState['voidTrader'], now: number): boolean {
	return Boolean(trader && !trader.completed && validDate(trader.activation)
		&& validDate(trader.expiry) && isCurrent(trader, now));
}

export type BaroCategory = 'mods' | 'weapons' | 'misc';

const weaponCategories = new Set([
	'primary', 'secondary', 'melee', 'arch-gun', 'arch-melee', 'sentinelweapons',
]);
export function baroCategory(metadata?: { category?: string | null; type?: string | null }): BaroCategory {
	const category = metadata?.category?.trim().toLowerCase() ?? '';
	const type = metadata?.type?.trim().toLowerCase() ?? '';
	if (category === 'mods' || type.endsWith(' mod') || type === 'mod' || type === 'stance') return 'mods';
	if (weaponCategories.has(category) || type === 'companion weapon' || type === 'sentinel weapon') return 'weapons';
	return 'misc';
}
