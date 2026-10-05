import { z } from 'zod';

// Fields used by shared item displays and the expanded Market card.
export const CatalogItemSchema = z.object({
	name: z.string().optional(),
	category: z.string().nullish(),
	tradable: z.boolean().optional(),
	marketSlug: z.string().nullish(),
	marketInfo: z.object({ urlName: z.string().nullish() }).nullish(),
	itemCount: z.number().int().nullish(),
	wikiaUrl: z.string().nullish(),
	description: z.string().nullish(),
	type: z.string().nullish(),
	rarity: z.string().nullish(),
	masteryReq: z.number().nullish(),
	compatName: z.string().nullish(),
	polarity: z.string().nullish(),
	baseDrain: z.number().nullish(),
	fusionLimit: z.number().nullish(),
	health: z.number().nullish(),
	shield: z.number().nullish(),
	armor: z.number().nullish(),
	power: z.number().nullish(),
	totalDamage: z.number().nullish(),
	criticalChance: z.number().nullish(),
	criticalMultiplier: z.number().nullish(),
	procChance: z.number().nullish(),
	magazineSize: z.number().nullish(),
	reloadTime: z.number().nullish(),
	buildPrice: z.number().nullish(),
	buildTime: z.number().nullish(),
	releaseDate: z.string().nullish(),
	levelStats: z.array(z.object({ stats: z.array(z.string()) })).nullish(),
});

export const MarketCatalogSchema = z.record(z.string(), CatalogItemSchema);
export type CatalogItem = z.infer<typeof CatalogItemSchema>;

const weaponCategories = new Set([
	'primary', 'secondary', 'melee', 'arch-gun', 'arch-melee', 'sentinelweapons',
]);

export function isWeaponCatalogItem(metadata?: { category?: string | null; type?: string | null }): boolean {
	const category = metadata?.category?.trim().toLowerCase() ?? '';
	const type = metadata?.type?.trim().toLowerCase() ?? '';
	return weaponCategories.has(category) || type === 'companion weapon' || type === 'sentinel weapon';
}

export function sanitizeItemDescription(text: string): string {
	return text.replaceAll('\\n', '\n').replace(/<[^>]*>/g, '').trim();
}

export function catalogMarketSlug(item?: { marketSlug?: string | null; marketInfo?: { urlName?: string | null } | null }) {
	return item?.marketSlug || item?.marketInfo?.urlName || undefined;
}

export function warframeItemName(item: CatalogItem | undefined, marketName: string | undefined, fallbackName: string): string {
	// /items component names are recipe labels (e.g. "Barrel"); /wfm-items names the complete item.
	if (item?.category === 'Components') return marketName || fallbackName || item.name || '';
	return item?.name || marketName || fallbackName;
}

export function itemMetadataDetails(item?: CatalogItem, fallbackMaxRank?: number) {
	const rows: { label: string; value: string }[] = [];
	if (!item) return rows;
	const number = new Intl.NumberFormat(undefined, { maximumFractionDigits: 2 });
	const add = (label: string, value: string | number | null | undefined, suffix = '') => {
		if (value == null || value === '' || (typeof value === 'number' && !Number.isFinite(value))) return;
		rows.push({ label, value: `${typeof value === 'number' ? number.format(value) : value}${suffix}` });
	};
	add('Type', item.type || item.category);
	add('Rarity', item.rarity);
	add('Mastery rank', item.masteryReq);
	add('Compatible with', item.compatName);
	add('Polarity', item.polarity);
	add('Base drain', item.baseDrain);
	add('Max rank', item.fusionLimit ?? fallbackMaxRank);
	add('Health', item.health);
	add('Shields', item.shield);
	add('Armor', item.armor);
	add('Energy', item.power);
	add('Base damage', item.totalDamage);
	add('Critical chance', item.criticalChance == null ? null : item.criticalChance * 100, '%');
	add('Critical multiplier', item.criticalMultiplier, '×');
	add('Status chance', item.procChance == null ? null : item.procChance * 100, '%');
	add('Magazine', item.magazineSize);
	add('Reload', item.reloadTime, ' s');
	add('Build cost', item.buildPrice, ' credits');
	add('Build time', item.buildTime == null ? null : item.buildTime / 3600, ' h');
	add('Released', item.releaseDate);
	if (item.tradable != null) add('Tradeable', item.tradable ? 'Yes' : 'No');
	return rows;
}
