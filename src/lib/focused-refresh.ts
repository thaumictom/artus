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
	let nextRefreshAt = immediate ? 0 : Date.now() + intervalMs;
	let timer: ReturnType<typeof setTimeout> | undefined;

	const schedule = () => {
		clearTimeout(timer);
		timer = setTimeout(refreshIfDue, Math.max(0, nextRefreshAt - Date.now()));
	};

	const run = async (onlyIfDue: boolean) => {
		if (
			disposed ||
			inFlight ||
			(onlyIfDue &&
				(!document.hasFocus() || document.hidden || Date.now() < nextRefreshAt))
		) {
			return;
		}

		inFlight = true;
		clearTimeout(timer);
		try {
			await callback();
		} finally {
			if (!disposed) {
				inFlight = false;
				nextRefreshAt = Date.now() + intervalMs;
				schedule();
			}
		}
	};

	function refreshIfDue() {
		void run(true);
	}

	window.addEventListener('focus', refreshIfDue);
	document.addEventListener('visibilitychange', refreshIfDue);
	if (immediate) refreshIfDue();
	else schedule();

	return {
		refresh: () => run(false),
		destroy: () => {
			disposed = true;
			clearTimeout(timer);
			window.removeEventListener('focus', refreshIfDue);
			document.removeEventListener('visibilitychange', refreshIfDue);
		},
	};
}
