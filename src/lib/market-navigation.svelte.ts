export type MarketNavigationTarget = {
	id: number;
	slug: string;
	since: number;
	orderType: 'buy' | 'sell';
};

export const marketNavigation = $state({
	target: null as MarketNavigationTarget | null,
});

export function openMarketNotificationTarget(
	slug: string,
	since: number,
	orderType: 'buy' | 'sell',
) {
	marketNavigation.target = {
		id: Date.now() + Math.random(),
		slug,
		since,
		orderType,
	};
}

export function clearMarketNotificationTarget() {
	marketNavigation.target = null;
}
