<script lang="ts">
	import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
	import ArtusMainPage from './ArtusMainPage.svelte';
	import ArtusSidebar from './ArtusSidebar.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import SiteHeader from './SiteHeader.svelte';
	import { mode } from 'mode-watcher';

	type ArtusSection = 'dashboard' | 'settings' | 'mastery' | 'inventory';

	const sections: { id: ArtusSection; label: string; icon: string }[] = [
		{ id: 'dashboard', label: 'Dashboard', icon: 'mdi:view-dashboard-outline' },
		{ id: 'mastery', label: 'Mastery', icon: 'mdi:star-outline' },
		{ id: 'inventory', label: 'Inventory', icon: 'mdi:package-variant-closed' },
		{ id: 'settings', label: 'Settings', icon: 'mdi:gear-outline' },
	];

	let activeSection = $state(sections[0]);
	let isSidebarOpen = $state(false);
	let scrollbarTheme = $derived(mode.current === 'dark' ? 'os-theme-light' : 'os-theme-dark');
</script>

<Sidebar.Provider bind:open={isSidebarOpen} class="h-svh">
	<ArtusSidebar {sections} bind:activeSection bind:isOpen={isSidebarOpen} />

	<Sidebar.Inset class="bg-sidebar">
		<SiteHeader title={activeSection.label} />
		<OverlayScrollbarsComponent
			defer
			options={{ scrollbars: { theme: scrollbarTheme } }}
			class="bg-background rounded-tl-[6px] h-full"
		>
			<ArtusMainPage activeSection={activeSection.id} />
		</OverlayScrollbarsComponent>
	</Sidebar.Inset>
</Sidebar.Provider>
