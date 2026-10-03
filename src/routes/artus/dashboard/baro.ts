import type { WorldState } from 'warframe-worldstate-parser';
import { isCurrent, validDate } from './views/view-types';

export function isBaroActive(trader: WorldState['voidTrader'], now: number): boolean {
	return Boolean(trader && !trader.completed && validDate(trader.activation)
		&& validDate(trader.expiry) && isCurrent(trader, now));
}

export type BaroCategory = 'mods' | 'weapons' | 'appearance' | 'misc';

const weaponCategories = new Set([
	'primary', 'secondary', 'melee', 'arch-gun', 'arch-melee', 'sentinelweapons',
]);
const appearanceCategories = new Set(['skins', 'glyphs', 'sigils']);
const appearanceTypes = new Set([
	'skin', 'skins', 'armor', 'armour', 'syandana', 'color palette', 'colour palette',
	'glyph', 'sigil', 'ship decoration', 'decoration', 'captura', 'emote', 'emotes',
	'fur color', 'fur pattern', 'pet collar', 'theme background', 'theme sound', 'themes',
]);

export function baroCategory(metadata?: { category?: string | null; type?: string | null }): BaroCategory {
	const category = metadata?.category?.trim().toLowerCase() ?? '';
	const type = metadata?.type?.trim().toLowerCase() ?? '';
	if (category === 'mods' || type.endsWith(' mod') || type === 'mod' || type === 'stance') return 'mods';
	if (weaponCategories.has(category) || type === 'companion weapon' || type === 'sentinel weapon') return 'weapons';
	if (appearanceCategories.has(category) || appearanceTypes.has(type)) return 'appearance';
	return 'misc';
}
