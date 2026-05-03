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
	import { Tabs } from 'bits-ui';
	import MainContent from './MainContent.svelte';
	import Header from './Header.svelte';
	import type { Sections } from '$lib/types';
	import AlertDialog from '$lib/components/AlertDialog.svelte';

	import Button from '$lib/components/Button.svelte';

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

	let showUpdatePrompt = $derived(Boolean(updateVersion) && !dismissedUpdatePrompt);

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
	<Header title={sections[activeSection].label}></Header>
	<AlertDialog bind:open={showUpdatePrompt}>
		{#snippet title()}
			<div>{updateVersion ?? 'The next version'} is ready to install</div>
		{/snippet}
		{#snippet description()}
			<div class="flex flex-col gap-2">
				<div>
					Please update at your earliest convenience. If you cancel now, press Ctrl+R to get
					prompted again.
				</div>
				{#if updateInstallError}
					<div class="text-danger">Error: {updateInstallError}</div>
				{/if}
			</div>
		{/snippet}
		{#snippet dialogCancel()}
			<Button
				onclick={continueWithoutUpdating}
				disabled={isInstallingUpdate}
				variant="default"
				tabindex={-1}
			>
				Cancel
			</Button>
		{/snippet}
		{#snippet dialogAction()}
			<Button
				onclick={downloadAndRelaunch}
				disabled={isInstallingUpdate}
				variant="primary"
				tabindex={-1}
			>
				{isInstallingUpdate ? 'Downloading...' : 'Download update and relaunch'}
			</Button>
		{/snippet}
	</AlertDialog>
	<Tabs.Root class="flex flex-1 overflow-hidden" orientation="vertical" bind:value={activeSection}>
		<div>
			<Sidebar {sections}></Sidebar>
		</div>
		<MainContent>
			<CurrentComponent />
		</MainContent>
	</Tabs.Root>
</div>
