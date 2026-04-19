<script lang="ts">
	import SettingsMain from '$lib/components/dashboard/SettingsMain.svelte';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import DashboardMain from './dashboard/Main.svelte';
	import InventoryTab from './inventory/Tab.svelte';
	import MasteryMain from './mastery/Main.svelte';
</script>

<Tabs.Root value="dashboard" class="w-full">
	<Tabs.List>
		<Tabs.Trigger value="dashboard">Dashboard</Tabs.Trigger>
		<Tabs.Trigger value="settings">Settings</Tabs.Trigger>
		<Tabs.Trigger value="mastery">Mastery</Tabs.Trigger>
		<Tabs.Trigger value="inventory">Inventory</Tabs.Trigger>
	</Tabs.List>
	<Tabs.Content value="dashboard" class="mt-4">
		<DashboardMain />
	</Tabs.Content>
	<Tabs.Content value="settings" class="mt-4">
		<SettingsMain />
	</Tabs.Content>
	<Tabs.Content value="mastery" class="mt-4">
		<MasteryMain />
	</Tabs.Content>
	<Tabs.Content value="inventory" class="mt-4">
		<InventoryTab />
	</Tabs.Content>
</Tabs.Root>
