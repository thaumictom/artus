import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';

const bountySchema = z.object({
	node: z.string().min(1),
	missionType: z.string().min(1),
	challenge: z.string().min(1),
	objective: z.string(),
	ally: z.string().nullable(),
	faction: z.string().nullable(),
});
const snapshotSchema = z.object({
	expiry: z.number().nullable(),
	bounties: z.record(z.string(), z.array(bountySchema)),
	error: z.string().nullable(),
});
type Snapshot = z.infer<typeof snapshotSchema>;

export const oracleBounties = $state({
	snapshot: null as Snapshot | null,
	loading: false,
	error: null as string | null,
});
let pending: Promise<void> | null = null;

export function reloadOracleBounties(): Promise<void> {
	if (pending) return pending;
	oracleBounties.loading = true;
	pending = (async () => {
		try {
			const snapshot = snapshotSchema.parse(await invoke('get_oracle_bounties'));
			oracleBounties.snapshot = snapshot;
			oracleBounties.error = snapshot.error;
		} catch (error) {
			console.error('Could not load Oracle bounties:', error);
			oracleBounties.error = 'Could not load current bounties.';
		} finally {
			oracleBounties.loading = false;
			pending = null;
		}
	})();
	return pending;
}
