<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { onMount, tick } from 'svelte';
	import { MediaQuery } from 'svelte/reactivity';
	import { initializeWorldState, reloadWorldState } from '$lib/worldstate.svelte';
	import {
		hasActiveNotificationRules,
		initializeNotificationCenter,
	} from '$lib/notifications.svelte';
	import { config, loadSettings, updateSetting } from '$lib/settings.svelte';
	import { loadMarketSession } from '$lib/market-account.svelte';
	import { ocrDebug } from '$lib/ocr-debug.svelte';
	import { initializeMarketNotifications } from '$lib/market-notifications.svelte';
	import { marketNavigation, openMarketNotificationTarget } from '$lib/market-navigation.svelte';
	import {
		appNavigation,
		navigateBack,
		navigateForward,
		navigateTo,
	} from '$lib/app-navigation.svelte';
	import { initializeMastery, stopMasteryListener } from '$lib/mastery.svelte';
	import { initializeWarframeItems } from '$lib/warframe-item.svelte';
	// import ArtusMainPage from './ArtusMainPage.svelte';
	// import ArtusSidebar from './ArtusSidebar.svelte';
	// import SiteHeader from './SiteHeader.svelte';
	import SettingsMain from './settings/Main.svelte';
	import DashboardMain from './dashboard/Main.svelte';
	import { dashboardViews } from './dashboard/dashboard-views';
	import InventoryTab from './inventory/Main.svelte';
	import MarketMain from './market/Main.svelte';
	import Listings from './listings/Main.svelte';
	import { marketAccount } from '$lib/market-account.svelte';
	import MasteryMain from './mastery/Main.svelte';

	import Sidebar from './Sidebar.svelte';
	import { Tabs } from 'bits-ui';
	import MainContent from './MainContent.svelte';
	import Header from './Header.svelte';
	import type { Sections } from '$lib/types';
	import AlertDialog from '$lib/components/AlertDialog.svelte';

	import Button from '$lib/components/Button.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';

	type UpdateAvailablePayload = {
		version: string;
	};
	type OcrDebugImagePayload = {
		png_bytes: number[];
		width: number;
		height: number;
	};

	const sections: Sections = {
		dashboard: {
			label: 'World State',
			icon: 'material-symbols:public',
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
		listings: {
			label: 'Listings',
			icon: 'material-symbols:format-list-bulleted-rounded',
			component: Listings,
		},
		settings: {
			label: 'Settings',
			icon: 'material-symbols:settings-outline-rounded',
			component: SettingsMain,
		},
	};

	const isNarrowViewport = new MediaQuery('(width < 800px)');
	let settingsReady = $state(false);
	let isSidebarOpen = $derived(!isNarrowViewport.current && config.sidebar_open);

	function toggleSidebar() {
		if (!settingsReady || isNarrowViewport.current) return;
		config.sidebar_open = !config.sidebar_open;
		void updateSetting('sidebar_open').catch((error) => {
			console.error('Could not save sidebar state:', error);
		});
	}
	let activeSection = $derived(appNavigation.current.section);
	let readyTableSection = $state<string | null>(null);
	$effect(() => {
		const section = activeSection;
		if (section !== 'mastery' && section !== 'inventory') return;
		readyTableSection = null;
		let timer: ReturnType<typeof setTimeout> | undefined;
		const frame = requestAnimationFrame(() => {
			timer = setTimeout(() => {
				readyTableSection = section;
			}, 0);
		});
		return () => {
			cancelAnimationFrame(frame);
			clearTimeout(timer);
		};
	});
	let dashboardViewLabel = $derived(
		dashboardViews.find((view) => view.value === appNavigation.current.dashboardView)?.label,
	);
	let handledMarketNavigationId: number | null = null;
	const CurrentComponent = $derived.by(() => sections[activeSection].component);

	let updateVersion: string | null = $state(null);
	let dismissedUpdatePrompt = $state(false);
	let updateInstallError: string | null = $state(null);
	let isInstallingUpdate = $state(false);

	let showUpdatePrompt = $derived(Boolean(updateVersion) && !dismissedUpdatePrompt);
	const NOTIFICATION_REFRESH_INTERVAL_MS = 3 * 60_000;

	onMount(() => {
		let disposed = false;
		let unlistenDebug: (() => void) | undefined;
		let stopWarframeItems: (() => void) | undefined;
		let startupTimer: ReturnType<typeof setTimeout> | undefined;
		let updateTimer: ReturnType<typeof setTimeout> | undefined;
		void listen<OcrDebugImagePayload>('ocr_debug_image', ({ payload }) => {
			if (disposed) return;
			if (ocrDebug.imageUrl) URL.revokeObjectURL(ocrDebug.imageUrl);
			ocrDebug.imageUrl = URL.createObjectURL(
				new Blob([new Uint8Array(payload.png_bytes)], { type: 'image/png' }),
			);
			ocrDebug.width = payload.width;
			ocrDebug.height = payload.height;
		})
			.then((unlisten) => {
				if (disposed) unlisten();
				else unlistenDebug = unlisten;
			})
			.catch((error) => console.error('Could not listen for OCR debug images:', error));
		// Side buttons are reported as buttons 3 (Back) and 4 (Forward).
		function preventSideButtonDefault(event: MouseEvent) {
			if (event.button === 3 || event.button === 4) event.preventDefault();
		}
		function handleSideButton(event: MouseEvent) {
			if (event.button === 3) navigateBack();
			else if (event.button === 4) navigateForward();
			else return;
			event.preventDefault();
		}
		window.addEventListener('mousedown', preventSideButtonDefault, true);
		window.addEventListener('mouseup', handleSideButton, true);
		window.addEventListener('auxclick', preventSideButtonDefault, true);
		void loadSettings()
			.catch((error) => console.error('Could not load settings:', error))
			.finally(() => {
				if (!disposed) settingsReady = true;
			});
		// Let the shell paint before starting independent catalog, account and notification work.
		const startupFrame = requestAnimationFrame(() => {
			startupTimer = setTimeout(() => {
				if (disposed) return;
				// World state must stay independent of settings-store initialization.
				initializeWorldState();
				stopWarframeItems = initializeWarframeItems();
				void loadMarketSession().catch((error) =>
					console.error('Could not load market session:', error),
				);
				void initializeNotificationCenter();
				void initializeMarketNotifications();
				void initializeMastery();
				updateTimer = setTimeout(() => {
					void checkForUpdate();
				}, 2000);
			}, 0);
		});
		return () => {
			disposed = true;
			cancelAnimationFrame(startupFrame);
			clearTimeout(startupTimer);
			clearTimeout(updateTimer);
			unlistenDebug?.();
			if (ocrDebug.imageUrl) URL.revokeObjectURL(ocrDebug.imageUrl);
			ocrDebug.imageUrl = null;
			window.removeEventListener('mousedown', preventSideButtonDefault, true);
			window.removeEventListener('mouseup', handleSideButton, true);
			window.removeEventListener('auxclick', preventSideButtonDefault, true);
			stopMasteryListener();
			stopWarframeItems?.();
		};
	});

	$effect(() => {
		if (!hasActiveNotificationRules()) return;
		const timer = setInterval(() => void reloadWorldState(), NOTIFICATION_REFRESH_INTERVAL_MS);
		return () => clearInterval(timer);
	});

	$effect(() => {
		if (marketAccount.ready && !marketAccount.session && activeSection === 'listings')
			navigateTo('market');
	});

	$effect(() => {
		const target = marketNavigation.target;
		if (!target || target.id === handledMarketNavigationId) return;
		handledMarketNavigationId = target.id;
		navigateTo('market', target.slug);
	});

	function openMarket(slug: string) {
		openMarketNotificationTarget(slug, Date.now(), 'sell');
		navigateTo('market', slug);
	}

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

	async function openNotificationSettings() {
		navigateTo('settings');
		await tick();
		document.getElementById('notifications')?.scrollIntoView({ block: 'start' });
	}
</script>

{#snippet tableSkeleton()}
	<div
		role="status"
		aria-label="Loading table"
		class="flex flex-col gap-4 mx-auto p-8 w-full max-w-5xl page-width"
	>
		<Skeleton class="w-full h-20" />
		<div class="flex flex-wrap gap-3">
			<Skeleton class="flex-1 min-w-56 h-10" />
			{#each Array(3) as _}<Skeleton class="w-36 h-10" />{/each}
		</div>
		<div class="border border-border-secondary divide-border-secondary divide-y">
			{#each Array(6) as _}<Skeleton class="w-full h-14" />{/each}
		</div>
	</div>
{/snippet}

<div class="flex flex-col bg-surface h-full artus-app-shell">
	<Header
		{isSidebarOpen}
		title={sections[activeSection].label}
		subtitle={activeSection === 'dashboard' ? dashboardViewLabel : undefined}
		onOpenNotificationSettings={openNotificationSettings}
	></Header>
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
	<Tabs.Root
		class="flex flex-1 overflow-hidden"
		orientation="vertical"
		value={activeSection}
		onValueChange={(value) => navigateTo(value)}
	>
		<Sidebar
			{sections}
			{isSidebarOpen}
			canToggle={settingsReady && !isNarrowViewport.current}
			onToggle={toggleSidebar}
		></Sidebar>
		{#if activeSection === 'dashboard'}
			<DashboardMain />
		{:else}
			<MainContent>
				{#if activeSection === 'mastery'}
					{#if readyTableSection === 'mastery'}
						<MasteryMain onOpenMarket={openMarket} />
					{:else}{@render tableSkeleton()}{/if}
				{:else if activeSection === 'inventory'}
					{#if readyTableSection === 'inventory'}
						<InventoryTab onOpenMarket={openMarket} />
					{:else}{@render tableSkeleton()}{/if}
				{:else if activeSection === 'listings'}
					<Listings onOpenMarket={openMarket} />
				{:else}
					<CurrentComponent />
				{/if}
			</MainContent>
		{/if}
	</Tabs.Root>
</div>
