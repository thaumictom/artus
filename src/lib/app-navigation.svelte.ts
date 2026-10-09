import type { DashboardView } from '../routes/artus/dashboard/dashboard-views';

export type AppLocation = {
	section: string;
	marketSlug: string;
	dashboardView: DashboardView;
};

const initialLocation: AppLocation = {
	section: 'dashboard',
	marketSlug: '',
	dashboardView: 'Home',
};

export const appNavigation = $state({
	entries: [initialLocation] as AppLocation[],
	index: 0,
	current: initialLocation,
});

export function navigateTo(
	section: string,
	marketSlug = '',
	dashboardView: DashboardView = appNavigation.current.dashboardView,
) {
	const current = appNavigation.current;
	if (
		current.section === section &&
		current.marketSlug === marketSlug &&
		current.dashboardView === dashboardView
	) return;
	const next: AppLocation = { section, marketSlug, dashboardView };
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
