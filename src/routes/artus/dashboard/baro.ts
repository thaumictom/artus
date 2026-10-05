import type { WorldState } from 'warframe-worldstate-parser';
import { isCurrent, validDate } from './views/view-types';
import { isWeaponCatalogItem } from '$lib/market-catalog';

export function isBaroActive(trader: WorldState['voidTrader'], now: number): boolean {
	return Boolean(trader && !trader.completed && validDate(trader.activation)
		&& validDate(trader.expiry) && isCurrent(trader, now));
}

export type BaroCategory = 'mods' | 'weapons' | 'misc';

export function baroCategory(metadata?: { category?: string | null; type?: string | null }): BaroCategory {
	const category = metadata?.category?.trim().toLowerCase() ?? '';
	const type = metadata?.type?.trim().toLowerCase() ?? '';
	if (category === 'mods' || type.endsWith(' mod') || type === 'mod' || type === 'stance') return 'mods';
	if (isWeaponCatalogItem(metadata)) return 'weapons';
	return 'misc';
}
