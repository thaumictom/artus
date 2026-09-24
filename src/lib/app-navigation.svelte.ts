export type AppLocation = {
	section: string;
	marketSlug: string;
};

const initialLocation: AppLocation = { section: 'dashboard', marketSlug: '' };

export const appNavigation = $state({
	entries: [initialLocation] as AppLocation[],
	index: 0,
	current: initialLocation,
});

export function navigateTo(section: string, marketSlug = '') {
	const current = appNavigation.current;
	if (current.section === section && current.marketSlug === marketSlug) return;
	const next = { section, marketSlug };
	appNavigation.entries = [...appNavigation.entries.slice(0, appNavigation.index + 1), next];
	appNavigation.index += 1;
	appNavigation.current = next;
}

export function navigateBack() {
	if (appNavigation.index === 0) return;
	appNavigation.index -= 1;
	appNavigation.current = appNavigation.entries[appNavigation.index];
}

export function navigateForward() {
	if (appNavigation.index === appNavigation.entries.length - 1) return;
	appNavigation.index += 1;
	appNavigation.current = appNavigation.entries[appNavigation.index];
}
