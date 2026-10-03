import type { WorldState } from 'warframe-worldstate-parser';
import { isCurrent, validDate } from './views/view-types';

export function isBaroActive(trader: WorldState['voidTrader'], now: number): boolean {
	return Boolean(trader && !trader.completed && validDate(trader.activation)
		&& validDate(trader.expiry) && isCurrent(trader, now));
}
