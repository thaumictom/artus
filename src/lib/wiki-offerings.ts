import { z } from 'zod';

export type WikiSource = 'tenet' | 'coda' | 'acrithis';
export const wikiSources = {
	tenet: { title: 'Tenet Weapons', url: 'https://wiki.warframe.com/w/Tenet_Weapons', seed: Date.UTC(2015, 11, 3), days: 4 },
	coda: { title: 'Coda Weapons', url: 'https://wiki.warframe.com/w/Coda_Weapons', seed: Date.UTC(2025, 2, 18), days: 4 },
	acrithis: { title: 'Acrithis', url: 'https://wiki.warframe.com/w/Acrithis', seed: Date.UTC(1970, 0, 5), days: 7 },
} as const;
export const wikiOfferingsSchema = z.object({
	items: z.array(z.object({ name: z.string(), element: z.string().nullable(), bonus: z.number().nullable() })),
	fetchedAt: z.number().nullable(), attemptedAt: z.number().nullable(),
	observedAt: z.number().nullable(), pageUpdatedAt: z.number().nullable(),
	reportedBatch: z.string().nullable(), error: z.string().nullable(),
	batches: z.record(z.string(), z.array(z.string())),
});
export type WikiOfferings = z.infer<typeof wikiOfferingsSchema>;
export type WikiOffering = WikiOfferings['items'][number];
export const UTC_DAY = 86_400_000;
export const WIKI_RETRY_INTERVAL = 5 * 60_000;

export function wikiRotation(source: WikiSource, now: number) {
	const { seed, days } = wikiSources[source];
	const cycle = Math.floor((now - seed) / (days * UTC_DAY));
	const start = seed + cycle * days * UTC_DAY;
	return { start, end: start + days * UTC_DAY, batch: cycle % 2 === 0 ? 'A' : 'B' };
}
