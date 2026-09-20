type FocusedRefreshOptions = {
	immediate?: boolean;
};

export function createFocusedRefresh(
	callback: () => void | Promise<void>,
	intervalMs: number,
	{ immediate = false }: FocusedRefreshOptions = {},
) {
	let disposed = false;
	let inFlight = false;
	let manualRefreshRequested = false;
	let nextIntervalRefreshAt = immediate ? 0 : Date.now() + intervalMs;
	let timer: ReturnType<typeof setTimeout> | undefined;

	const hasFocus = () => document.hasFocus() && !document.hidden;

	const nextRefreshAt = () => (manualRefreshRequested ? 0 : nextIntervalRefreshAt);

	const schedule = () => {
		clearTimeout(timer);
		if (disposed || inFlight) return;

		const refreshAt = nextRefreshAt();
		// Focus/visibility events will resume an overdue refresh without polling in the background.
		if (refreshAt <= Date.now() && !hasFocus()) return;
		timer = setTimeout(refreshIfDue, Math.max(0, refreshAt - Date.now()));
	};

	const runIfDue = async () => {
		const now = Date.now();
		if (disposed || inFlight || !hasFocus() || now < nextRefreshAt()) return;

		inFlight = true;
		manualRefreshRequested = false;
		clearTimeout(timer);
		try {
			await callback();
		} finally {
			if (!disposed) {
				inFlight = false;
				nextIntervalRefreshAt = Date.now() + intervalMs;
				schedule();
			}
		}
	};

	function refreshIfDue() {
		void runIfDue();
	}

	window.addEventListener('focus', refreshIfDue);
	document.addEventListener('visibilitychange', refreshIfDue);
	if (immediate) refreshIfDue();
	else schedule();

	return {
		refresh: () => {
			if (disposed) return Promise.resolve();
			manualRefreshRequested = true;
			return runIfDue();
		},
		destroy: () => {
			disposed = true;
			clearTimeout(timer);
			window.removeEventListener('focus', refreshIfDue);
			document.removeEventListener('visibilitychange', refreshIfDue);
		},
	};
}
