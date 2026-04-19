<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import SettingsMain from '$lib/components/dashboard/SettingsMain.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import DashboardMain from './dashboard/Main.svelte';
	import InventoryTab from './inventory/Tab.svelte';
	import MasteryMain from './mastery/Main.svelte';
	import Icon from '@iconify/svelte';

	type ArtusSection = 'dashboard' | 'settings' | 'mastery' | 'inventory';

	const sections: { id: ArtusSection; label: string; icon: string }[] = [
		{ id: 'dashboard', label: 'Dashboard', icon: 'lucide:home' },
		{ id: 'mastery', label: 'Mastery', icon: 'lucide:star' },
		{ id: 'inventory', label: 'Inventory', icon: 'lucide:package' },
		{ id: 'settings', label: 'Settings', icon: 'lucide:settings' },
	];

	let activeSection = $state<ArtusSection>('dashboard');
	let isSidebarOpen = $state(false);

	const appWindow = getCurrentWindow();

	async function withWindowAction(action: () => Promise<void>) {
		try {
			await action();
		} catch {
			// Ignore when not running inside a Tauri window.
		}
	}

	function minimizeWindow() {
		void withWindowAction(() => appWindow.minimize());
	}

	function toggleMaximizeWindow() {
		void withWindowAction(() => appWindow.toggleMaximize());
	}

	function closeWindow() {
		void withWindowAction(() => appWindow.close());
	}
</script>

<div class="flex flex-col w-full h-full min-h-0">
	<header class="flex items-center bg-sidebar border-sidebar-border border-b h-9 shrink-0">
		<div class="flex flex-1 items-center px-3 min-w-0" data-tauri-drag-region>
			<p class="font-medium text-sidebar-foreground text-xs truncate uppercase tracking-wide">
				Artus
			</p>
		</div>
		<div class="flex items-center gap-0.5 pr-1">
			<button
				type="button"
				class="hover:bg-sidebar-accent p-1.5 rounded-none text-sidebar-foreground/80 hover:text-sidebar-foreground transition-colors"
				onclick={minimizeWindow}
				aria-label="Minimize window"
			>
				<Icon icon="lucide:minus" class="text-base" aria-hidden="true" />
			</button>
			<button
				type="button"
				class="hover:bg-sidebar-accent p-1.5 rounded-none text-sidebar-foreground/80 hover:text-sidebar-foreground transition-colors"
				onclick={toggleMaximizeWindow}
				aria-label="Toggle maximize"
			>
				<Icon icon="lucide:square" class="text-sm" aria-hidden="true" />
			</button>
			<button
				type="button"
				class="hover:bg-destructive/80 p-1.5 rounded-none text-sidebar-foreground/80 hover:text-destructive-foreground transition-colors"
				onclick={closeWindow}
				aria-label="Close window"
			>
				<Icon icon="lucide:x" class="text-base" aria-hidden="true" />
			</button>
		</div>
	</header>

	<Sidebar.Provider bind:open={isSidebarOpen} class="flex-1 w-full min-h-0 overflow-visible">
		<Sidebar.Root
			collapsible="icon"
			class="border-sidebar-border border-r"
			onmouseenter={() => (isSidebarOpen = true)}
			onmouseleave={() => (isSidebarOpen = false)}
		>
			<Sidebar.Content>
				<Sidebar.Group>
					<Sidebar.GroupContent>
						<Sidebar.Menu>
							{#each sections as section (section.id)}
								<Sidebar.MenuItem>
									<Sidebar.MenuButton
										isActive={activeSection === section.id}
										onclick={() => (activeSection = section.id)}
									>
										<Icon icon={section.icon} class="text-lg" aria-hidden="true"></Icon>
										<span>{section.label}</span>
									</Sidebar.MenuButton>
								</Sidebar.MenuItem>
							{/each}
						</Sidebar.Menu>
					</Sidebar.GroupContent>
				</Sidebar.Group>
			</Sidebar.Content>
		</Sidebar.Root>

		<Sidebar.Inset
			class="md:flex-none! md:w-[calc(100%-var(--sidebar-width-icon))]! min-w-0 overflow-auto transition-transform duration-200 ease-linear"
		>
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
		</Sidebar.Inset>
	</Sidebar.Provider>
</div>
