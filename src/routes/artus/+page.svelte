<script lang="ts">
	import ArtusMainPage from './ArtusMainPage.svelte';
	import ArtusSidebar from './ArtusSidebar.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import SiteHeader from './SiteHeader.svelte';

	type ArtusSection = 'dashboard' | 'settings' | 'mastery' | 'inventory';

	const sections: { id: ArtusSection; label: string; icon: string }[] = [
		{ id: 'dashboard', label: 'Dashboard', icon: 'lucide:home' },
		{ id: 'mastery', label: 'Mastery', icon: 'lucide:star' },
		{ id: 'inventory', label: 'Inventory', icon: 'lucide:package' },
		{ id: 'settings', label: 'Settings', icon: 'lucide:settings' },
	];

	let activeSection = $state(sections[0]);
	let isSidebarOpen = $state(false);
</script>

<div class="bg-sidebar">
	<div data-tauri-drag-region class="bg-sidebar h-2"></div>
	<Sidebar.Provider bind:open={isSidebarOpen}>
		<ArtusSidebar {sections} bind:activeSection bind:isOpen={isSidebarOpen} />

		<Sidebar.Inset class="mr-2 rounded-sm overflow-hidden">
			<SiteHeader title={activeSection.label} />
			<ArtusMainPage activeSection={activeSection.id} />
		</Sidebar.Inset>
	</Sidebar.Provider>
</div>
