<script lang="ts">
	import SettingsMain from './settings/Main.svelte';
	import DashboardMain from './dashboard/Main.svelte';
	import InventoryTab from './inventory/Tab.svelte';
	import MasteryMain from './mastery/Main.svelte';

	type ArtusSection = 'dashboard' | 'settings' | 'mastery' | 'inventory';

	let {
		activeSection,
	}: {
		activeSection: ArtusSection;
	} = $props();
</script>

<div class="p-4">
	{#if activeSection === 'dashboard'}
		<DashboardMain />
	{:else if activeSection === 'settings'}
		<SettingsMain />
	{:else if activeSection === 'mastery'}
		<MasteryMain />
	{:else}
		<InventoryTab />
	{/if}
</div>
