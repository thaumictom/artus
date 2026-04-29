<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	// import ArtusMainPage from './ArtusMainPage.svelte';
	// import ArtusSidebar from './ArtusSidebar.svelte';
	// import SiteHeader from './SiteHeader.svelte';
	import SettingsMain from './settings/Main.svelte';
	import DashboardMain from './dashboard/Main.svelte';
	import InventoryTab from './inventory/Tab.svelte';
	import MarketMain from './market/Main.svelte';
	import MasteryMain from './mastery/Main.svelte';

	import Sidebar from './Sidebar.svelte';
	import { Button, Tabs } from 'bits-ui';
	import MainContent from './MainContent.svelte';
	import Header from './Header.svelte';
	import type { Sections } from '$lib/types';

	type UpdateAvailablePayload = {
		version: string;
	};

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
		market: {
			label: 'Market',
			icon: 'material-symbols:shopping-cart-outline-rounded',
			component: MarketMain,
		},
		settings: {
			label: 'Settings',
			icon: 'material-symbols:settings-outline-rounded',
			component: SettingsMain,
		},
	};

	let activeSection = $state('dashboard');
	const CurrentComponent = $derived.by(() => sections[activeSection].component);

	let updateVersion: string | null = $state(null);
	let dismissedUpdatePrompt = $state(false);
	let updateInstallError: string | null = $state(null);
	let isInstallingUpdate = $state(false);

	const showUpdatePrompt = $derived(Boolean(updateVersion) && !dismissedUpdatePrompt);

	onMount(() => {
		void checkForUpdate();
	});

	async function checkForUpdate() {
		try {
			const update = await invoke<UpdateAvailablePayload | null>('check_for_update');
			updateVersion = update?.version ?? null;
			dismissedUpdatePrompt = false;
			updateInstallError = null;
		} catch (error) {
			console.error('[updater] failed to check for updates', error);
		}
	}

	async function downloadAndRelaunch() {
		if (isInstallingUpdate) return;

		isInstallingUpdate = true;
		updateInstallError = null;

		try {
			await invoke('download_and_relaunch_update');
		} catch (error) {
			isInstallingUpdate = false;
			updateInstallError = String(error);
		}
	}

	function continueWithoutUpdating() {
		dismissedUpdatePrompt = true;
	}
</script>

<div class="flex flex-col bg-surface h-full">
	<!-- <Header title={sections[activeSection].label}></Header> -->
	{#if showUpdatePrompt}
		<div
			class="flex sm:flex-row flex-col sm:justify-between sm:items-center gap-3 bg-elevated mx-3 mt-2 p-3 border border-accent/40 rounded-md text-sm"
		>
			<div class="min-w-0">
				<div class="font-medium text-accent">Update available!</div>
				<div class="text-muted-foreground">
					Version {updateVersion} is ready to install.
				</div>
				{#if updateInstallError}
					<div class="mt-1 text-danger">{updateInstallError}</div>
				{/if}
			</div>
			<div class="flex flex-wrap gap-2">
				<Button.Root
					onclick={downloadAndRelaunch}
					disabled={isInstallingUpdate}
					class="bg-accent hover:bg-accent/90 px-3 py-1 rounded text-accent-foreground"
					tabindex={-1}
				>
					{isInstallingUpdate ? 'Downloading...' : 'Download and relaunch'}
				</Button.Root>
				<Button.Root
					onclick={continueWithoutUpdating}
					disabled={isInstallingUpdate}
					class="bg-surface hover:bg-muted px-3 py-1 rounded"
					tabindex={-1}
				>
					Continue without updating
				</Button.Root>
			</div>
		</div>
	{/if}
	<Tabs.Root class="flex flex-1 overflow-hidden" orientation="vertical" bind:value={activeSection}>
		<div>
			<Sidebar {sections}></Sidebar>
		</div>
		<MainContent>
			<CurrentComponent />
		</MainContent>
	</Tabs.Root>
</div>
