import { z } from 'zod';

// Keep only the fields used by the card in the Market tab's in-memory copy.
export const CatalogItemSchema = z.object({
	name: z.string().optional(),
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
