import type { MasteryItem } from '$lib/mastery.svelte';

const NORMAL_RANK_CAP = 30;
const LEGENDARY_RANK_XP = 147_500;
const NORMAL_RANK_CAP_XP = 2_500 * NORMAL_RANK_CAP ** 2;

export function masteryXpFor(item: MasteryItem): number {
	const isFortyRankWeapon = /^(Kuva|Tenet|Coda) /.test(item.name) || item.name === 'Paracesis';
	if (item.type === 'Necramech') return 8_000;
	if (item.type === 'Warframe' || item.type === 'Sentinel' || item.type === 'Pets' || item.type === 'Archwing' || item.type === 'K-Drive Component') return 6_000;
	return isFortyRankWeapon ? 4_000 : 3_000;
}

export function masteryRankProgress(xp: number) {
	// Manual mastery XP can be any safe integer, so avoid stepping through every rank.
	const rank = xp < NORMAL_RANK_CAP_XP
		? Math.floor(Math.sqrt(xp / 2_500))
		: NORMAL_RANK_CAP + Math.floor((xp - NORMAL_RANK_CAP_XP) / LEGENDARY_RANK_XP);
	const rankStart = xpForRank(rank);
	const nextRank = xpForRank(rank + 1);
	return {
		label: rank <= NORMAL_RANK_CAP ? `MR ${rank}` : `LR ${rank - NORMAL_RANK_CAP}`,
		xp,
		current: xp - rankStart,
		required: nextRank - rankStart,
		percent: ((xp - rankStart) / (nextRank - rankStart)) * 100,
	};
}

function xpForRank(rank: number) {
	return rank <= NORMAL_RANK_CAP
		? 2_500 * rank ** 2
		: NORMAL_RANK_CAP_XP + (rank - NORMAL_RANK_CAP) * LEGENDARY_RANK_XP;
}
