import { invoke } from '@tauri-apps/api/core';
import 'reflect-metadata';
import { SyndicateJob, SyndicateMission, WorldEvent, WorldState, type InitialWorldState } from 'warframe-worldstate-parser';
import { processWorldStateNotifications } from '$lib/notifications.svelte';
import { reloadOracleBounties } from '$lib/oracle-bounties.svelte';

type RawWorldState = Omit<InitialWorldState, 'Events'> & {
	Events: (InitialWorldState['Events'][number] & { Community?: boolean })[];
};
export type ArtusWorldState = Omit<WorldState, 'news'> & {
	news: (WorldState['news'][number] & { community: boolean })[];
};

export const dashboard = $state({
	world: null as ArtusWorldState | null,
	fetchedAt: null as number | null,
	loading: false,
	error: null as string | null,
});
let started = false;
let pending: Promise<void> | null = null;

export function reloadWorldState(): Promise<void> {
	if (pending) return pending;
	// Oracle has its own shared rotation cache and must not delay or fail the world state.
	void reloadOracleBounties();
	dashboard.loading = true;
	dashboard.error = null;
	pending = (async () => {
		try {
			const raw = await invoke<RawWorldState>('get_world_state');
			const previousWorld = dashboard.world;
			const world = new WorldState(raw, { locale: 'en' });
			// The synchronous constructor leaves events empty; parse Goals locally without
			// the async builder's additional bounty reward requests.
			world.events = (raw.Goals ?? []).map((event) => new WorldEvent(event, { locale: 'en' }));
			world.syndicateMissions = (raw.SyndicateMissions ?? []).map((mission) => {
				const syndicate = new SyndicateMission(mission, { locale: 'en' });
				// Jobs are present in the raw world state; their external reward pools are not.
				const expiry = syndicate.expiry;
				if (expiry instanceof Date && Number.isFinite(expiry.getTime())) {
					syndicate.jobs = (mission.Jobs ?? []).map((job) =>
						new SyndicateJob(job, expiry, { locale: 'en' }));
				}
				return syndicate;
			});
			// The parser filters news by locale and drops Community; join by ID,
			// rather than array position, to preserve the API's classification.
			const communityById = new Map((raw.Events ?? []).flatMap((event) => {
				const id = event._id?.$oid || event._id?.$id;
				return id ? [[id, event.Community === true] as const] : [];
			}));
			dashboard.world = Object.assign(world, {
				news: world.news.map((article) => Object.assign(article, {
					community: Boolean(article.id && communityById.get(article.id)),
				})),
			});
			dashboard.fetchedAt = Date.now();
			await processWorldStateNotifications(world, previousWorld);
		} catch (error) {
			console.error('Could not load world state:', error);
			dashboard.error = 'Could not fetch world state. Check your connection and try Reload.';
		} finally {
			dashboard.loading = false;
			pending = null;
		}
	})();
	return pending;
}

export function initializeWorldState() {
	if (started) return;
	started = true;
	void reloadWorldState();
}
