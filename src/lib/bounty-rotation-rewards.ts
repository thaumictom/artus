export type BountyRotation = 'A' | 'B' | 'C';
export type BountyRotationKind = 'cetus' | 'vault';
type RotationReward = { name: string; slug?: string };

// Notable rewards, not full drop tables. Source:
// https://github.com/calamity-inc/browse.wf/blob/senpai/live.ts#L341-L351
export const bountyRotationRewards: Record<BountyRotationKind, Record<BountyRotation, readonly RotationReward[]>> = {
	cetus: {
		A: [{ name: 'Caliban Systems' }, { name: 'Verdilac', slug: 'verdilac' }],
		B: [{ name: 'Caliban Chassis' }, { name: 'Nepheri', slug: 'nepheri' }],
		C: [{ name: 'Caliban Neuroptics' }, { name: 'Korumm', slug: 'korumm' }],
	},
	vault: {
		A: [{ name: 'Sporothrix Barrel', slug: 'sporothrix_barrel' }],
		B: [{ name: 'Sporothrix Receiver', slug: 'sporothrix_receiver' }],
		C: [{ name: 'Sporothrix Stock', slug: 'sporothrix_stock' }],
	},
};
