import { LazyStore } from '@tauri-apps/plugin-store';
import { z } from 'zod';
import { ClanReward, type WorldState } from 'warframe-worldstate-parser';

const rewardSchema = z.object({
	RewardClaimed: z.boolean(), PointThreshold: z.number(), ItemCount: z.number(), Reward: z.string(),
});
const store = new LazyStore('clan-initiative-completions.json');
let pending: Promise<unknown> = Promise.resolve();
export const clanInitiativeCompletions = $state({ completedIds: [] as string[], loaded: false });
const completedIds = $derived(new Set(clanInitiativeCompletions.completedIds));

export function getClanInitiativeRewards(initiative: WorldState['clanWeeklyInitiative']) {
	const expiry = initiative?.expiry?.getTime();
	if (!initiative || expiry === undefined || !Number.isFinite(expiry)) return [];
	return (initiative.rewwards ?? []).flatMap((raw) => {
		const result = rewardSchema.safeParse(raw);
		if (!result.success) return [];
		const reward = new ClanReward(result.data, { locale: 'en' });
		// Threshold and rotation identify each reward, including repeated item rewards.
		const id = `${expiry}:${reward.pointsRequired}:${reward.uniqueName}:${reward.count}`;
		return [{ id, reward }];
	});
}

export function isClanInitiativeRewardCompleted(id: string): boolean {
	return completedIds.has(id);
}

function withStore<T>(action: () => Promise<T>): Promise<T> {
	const result = pending.then(action);
	pending = result.catch(() => {});
	return result;
}

export function initializeClanInitiativeCompletions(): Promise<void> {
	return withStore(async () => {
		if (clanInitiativeCompletions.loaded) return;
		clanInitiativeCompletions.completedIds = await store.get<string[]>('completedIds') ?? [];
		clanInitiativeCompletions.loaded = true;
	});
}

export function setClanInitiativeRewardsCompleted(ids: string[], completed: boolean): Promise<void> {
	return withStore(async () => {
		const saved = await store.get<string[]>('completedIds') ?? [];
		const selected = new Set(ids);
		const updated = completed ? [...new Set([...saved, ...ids])] : saved.filter((id) => !selected.has(id));
		await store.set('completedIds', updated);
		await store.save();
		clanInitiativeCompletions.completedIds = updated;
		clanInitiativeCompletions.loaded = true;
	});
}
