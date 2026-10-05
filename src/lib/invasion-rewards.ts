import type { Reward, WorldState } from 'warframe-worldstate-parser';

const excludedRewards = new Set([
	'fieldron', 'detonite injector', 'mutagen mass', 'mutalist alad v nav coordinate',
]);

export function isExcludedInvasionReward(name: string): boolean {
	return excludedRewards.has(name.trim().replace(/\s+/g, ' ').toLowerCase());
}

function rewardNames(reward?: Reward): string[] {
	return reward?.countedItems?.length
		? reward.countedItems.map((item) => item.type)
		: reward?.items ?? [];
}

export function hasInvasionReward(reward?: Reward): reward is Reward {
	return Boolean(reward && (rewardNames(reward).length > 0 || reward.credits > 0));
}

export function hasOnlyExcludedInvasionRewards(invasion: WorldState['invasions'][number]): boolean {
	// Keep invasions that offer any other item on either side.
	const names = [invasion.attacker.reward, invasion.defender.reward].flatMap(rewardNames);
	return names.length > 0 && names.every(isExcludedInvasionReward);
}

export function invasionRewardOpportunities(invasion: WorldState['invasions'][number]): Reward[] {
	// Each side is one opportunity, regardless of item quantity in its battle pay.
	return [invasion.attacker.reward, invasion.defender.reward]
		.filter(hasInvasionReward)
		.filter((reward) => {
			const names = rewardNames(reward);
			return names.length === 0 || names.some((name) => !isExcludedInvasionReward(name));
		});
}
