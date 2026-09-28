import { timeAgo } from '$lib/date';
import { createFocusedRefresh } from '$lib/focused-refresh';

const REFRESH_INTERVAL_MS = 60_000;
const MANUAL_RELOAD_COOLDOWN_MS = 3_000;

/** Focus-aware refresh state shared by market orders and the mastery buy dialog. */
export class MarketOrdersRefresh {
	refreshing = $state(false);
	coolingDown = $state(false);
	fetchedAt = $state<number | null>(null);
	now = $state(Date.now());
	fetchedAgo = $derived(this.fetchedAt === null ? '' : timeAgo(this.fetchedAt, this.now));

	private generation = 0;
	private forceNext = false;
	private cooldownTimer: ReturnType<typeof setTimeout> | undefined;
	private clockTimer: ReturnType<typeof setInterval> | undefined;
	private focusedRefresh: ReturnType<typeof createFocusedRefresh> | undefined;

	start(load: (forceRefresh: boolean) => Promise<boolean>) {
		this.destroy();
		const generation = this.generation;
		this.refreshing = false;
		this.coolingDown = false;
		this.fetchedAt = null;
		this.now = Date.now();
		this.clockTimer = setInterval(() => { this.now = Date.now(); }, 1000);
		this.focusedRefresh = createFocusedRefresh(async () => {
			this.refreshing = true;
			const forceRefresh = this.forceNext;
			this.forceNext = false;
			try {
				const fetched = await load(forceRefresh);
				if (generation === this.generation && fetched) this.fetchedAt = Date.now();
			} finally {
				if (generation === this.generation) this.refreshing = false;
			}
		}, REFRESH_INTERVAL_MS, { immediate: true });
	}

	reload() {
		if (!this.focusedRefresh || this.refreshing || this.coolingDown) return;
		this.coolingDown = true;
		this.cooldownTimer = setTimeout(() => { this.coolingDown = false; }, MANUAL_RELOAD_COOLDOWN_MS);
		this.forceNext = true;
		void this.focusedRefresh.refresh();
	}

	destroy() {
		this.generation++;
		this.focusedRefresh?.destroy();
		this.focusedRefresh = undefined;
		clearTimeout(this.cooldownTimer);
		clearInterval(this.clockTimer);
		this.forceNext = false;
	}
}
