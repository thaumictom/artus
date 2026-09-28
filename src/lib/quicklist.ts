export const quicklistStrategies = [
	{ value: 'median', label: 'Always set median' },
	{ value: 'below_median', label: '1 below median' },
	{ value: 'match_cheapest', label: 'Match cheapest' },
	{ value: 'undercut_cheapest', label: 'Undercut cheapest by 1' },
	{ value: 'undercut_above_median', label: 'Undercut cheapest over median' },
	{ value: 'undercut_at_or_above_median', label: 'Undercut cheapest over or on median' },
] as const;

export type QuicklistStrategy = (typeof quicklistStrategies)[number]['value'];
export const isQuicklistStrategy = (value: unknown): value is QuicklistStrategy =>
	quicklistStrategies.some((strategy) => strategy.value === value);

export function quicklistPrice(strategy: QuicklistStrategy, median: number | null, offers: number[]): number {
	const roundedMedian = median !== null && Number.isFinite(median) && median > 0 ? Math.round(median) : null;
	const prices = offers.filter((price) => Number.isSafeInteger(price) && price > 0).sort((a, b) => a - b);
	let price: number | null;
	switch (strategy) {
		case 'median': price = roundedMedian; break;
		case 'below_median': price = roundedMedian === null ? null : roundedMedian - 1; break;
		case 'match_cheapest': price = prices[0] ?? null; break;
		case 'undercut_cheapest': price = prices[0] === undefined ? null : prices[0] - 1; break;
		case 'undercut_above_median': {
			if (roundedMedian === null) { price = null; break; }
			const cheapestAbove = prices.find((offer) => offer > roundedMedian);
			price = cheapestAbove === undefined ? roundedMedian : cheapestAbove - 1;
			break;
		}
		case 'undercut_at_or_above_median': {
			if (roundedMedian === null) { price = null; break; }
			const cheapestAtOrAbove = prices.find((offer) => offer >= roundedMedian);
			price = cheapestAtOrAbove === undefined ? roundedMedian : cheapestAtOrAbove - 1;
			break;
		}
	}
	if (price === null) throw new Error('No market price is available for this strategy');
	if (price < 1 || price > 900000) throw new Error('Quicklist price must be between 1 and 900,000 platinum');
	return price;
}
