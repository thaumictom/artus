<script lang="ts">
	// import ArtusMainPage from './ArtusMainPage.svelte';
	// import ArtusSidebar from './ArtusSidebar.svelte';
	// import SiteHeader from './SiteHeader.svelte';
	import SettingsMain from './settings/Main.svelte';
	import DashboardMain from './dashboard/Main.svelte';
	import InventoryTab from './inventory/Tab.svelte';
	import MasteryMain from './mastery/Main.svelte';

	import Sidebar from './Sidebar.svelte';
	import { Tabs } from 'bits-ui';
	import MainContent from './MainContent.svelte';
	import Header from './Header.svelte';
	import type { Sections } from '$lib/types';

	const sections: Sections = {
		dashboard: {
			label: 'Dashboard',
			icon: 'material-symbols:space-dashboard-outline-rounded',
			component: DashboardMain,
		},
		mastery: {
			label: 'Mastery',
			icon: 'material-symbols:star-outline-rounded',
			component: MasteryMain,
		},
		inventory: {
			label: 'Inventory',
			icon: 'material-symbols:package-2-outline',
			component: InventoryTab,
		},
		settings: {
			label: 'Settings',
			icon: 'material-symbols:settings-outline-rounded',
			component: SettingsMain,
		},
	};

	let activeSection = $state('dashboard');
	const CurrentComponent = $derived.by(() => sections[activeSection].component);
</script>

<div class="flex flex-col bg-surface h-full">
	<Header title={sections[activeSection].label}></Header>
	<Tabs.Root class="flex flex-1 overflow-hidden" orientation="vertical" bind:value={activeSection}>
		<div>
			<Sidebar {sections}></Sidebar>
		</div>
		<MainContent>
			<CurrentComponent />
		</MainContent>
	</Tabs.Root>
</div>
