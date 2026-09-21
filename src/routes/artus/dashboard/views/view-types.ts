import type { Reward, WorldState } from 'warframe-worldstate-parser';
export type DashboardViewProps = {
	world: WorldState;
	now: number;
};
export type ViewRow = {
	title: string;
	description?: string;
	details?: string[];
	value?: string;
	expiry?: Date;
};
export function validDate(date?: Date): date is Date {
	return date instanceof Date && Number.isFinite(date.getTime());
}
export function isCurrent(item: {
	activation?: Date;
	expiry?: Date;
} | undefined, now: number) {
	return Boolean(item && (!validDate(item.activation) || item.activation.getTime() <= now)
		&& (!validDate(item.expiry) || item.expiry.getTime() > now));
}
export function amount(value: number | undefined, unit: string) {
	return value !== undefined && Number.isFinite(value) ? `${value.toLocaleString()} ${unit}` : `${unit}: unavailable`;
}
export function rewardText(reward?: Reward) {
	if (!reward)
		return 'Reward unavailable';
	return [
		...(reward.items ?? []),
		...(reward.countedItems ?? []).map((item) => `${item.count}× ${item.type}`),
		...(reward.credits > 0 ? [amount(reward.credits, 'credits')] : []),
	].join(' · ') || 'Reward unavailable';
}
