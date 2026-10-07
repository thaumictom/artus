<script lang="ts">
	import { Separator } from 'bits-ui';
	import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
	import WarframeSettings from './categories/WarframeSettings.svelte';
	import MetaInformation from './categories/MetaInformation.svelte';
	import OverlaySettings from './categories/OverlaySettings.svelte';
	import OverlayBehaviour from './categories/OverlayBehaviour.svelte';
	import DebugSettings from './categories/DebugSettings.svelte';
	import Maintenance from './categories/Maintenance.svelte';
	import NotificationSettings from './categories/NotificationSettings.svelte';
	import AppBehaviour from './categories/AppBehaviour.svelte';
	import Quicklist from './categories/Quicklist.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
	import { kebabCase } from 'change-case';
	import MainContent from '../MainContent.svelte';
	import ResizableNavigationLayout from '../ResizableNavigationLayout.svelte';

	const allSections = [
		{ name: 'App Behaviour', component: AppBehaviour },
		{ name: 'Notifications', component: NotificationSettings },
		{ name: 'Warframe Settings', component: WarframeSettings },
		{ name: 'Overlay Settings', component: OverlaySettings },
		{ name: 'Overlay Behaviour', component: OverlayBehaviour },
		{ name: 'Quicklist', component: Quicklist },
		{ name: 'Maintenance', component: Maintenance },
		{ name: 'Developer', component: DebugSettings },
		{ name: 'About', component: MetaInformation },
	].map((section) => ({ ...section, id: kebabCase(section.name) }));
	let sections = $derived(
		allSections.filter((section) => section.id !== 'developer' || config.show_debug_settings),
	);

	let activeSection = $state(allSections[0].id);

	function updateActiveSection(scrollElement: HTMLElement) {
		const firstHeading = document.getElementById(sections[0].id);
		if (!firstHeading) return;
		// Match the heading's scroll margin used by scrollIntoView.
		const scrollMargin = parseFloat(getComputedStyle(firstHeading).scrollMarginTop) || 0;
		const top = scrollElement.getBoundingClientRect().top + scrollMargin + 1;
		let current = sections[0].id;
		for (const section of sections) {
			const heading = document.getElementById(section.id);
			if (!heading || heading.getBoundingClientRect().top > top) break;
			current = section.id;
		}
		if (scrollElement.scrollTop + scrollElement.clientHeight >= scrollElement.scrollHeight - 2) {
			current = sections[sections.length - 1].id;
		}
		activeSection = current;
	}

	function scrollToSection(id: string) {
		activeSection = id;
		document.getElementById(id)?.scrollIntoView({ behavior: 'instant', block: 'start' });
	}

	function setDebugVisibility(visible: boolean) {
		if (config.show_debug_settings === visible) return;
		config.show_debug_settings = visible;
		void updateSetting('show_debug_settings');
	}
</script>

<ResizableNavigationLayout resizeLabel="Resize settings navigation">
	{#snippet navigation()}
		<nav
			aria-label="Settings sections"
			class="flex flex-col bg-background rounded-t-md w-full h-full min-h-0"
		>
			<OverlayScrollbarsComponent
				defer
				options={{ scrollbars: { theme: 'os-theme-light', autoHide: 'move' } }}
				class="flex-1 w-full min-h-0"
			>
				<div class="flex items-center p-3 pt-4 min-h-12">
					<h2
						class="px-3 py-1 font-semibold text-muted-foreground text-xs uppercase tracking-widest"
					>
						Settings
					</h2>
				</div>
				<ul class="flex flex-col gap-0.5 px-3">
					{#each sections as section (section.id)}
						<li>
							<button
								type="button"
								aria-current={activeSection === section.id ? 'location' : undefined}
								onclick={(event) => {
									if (section.id === 'developer' && event.detail === 2) {
										setDebugVisibility(false);
										scrollToSection('about');
										return;
									}
									scrollToSection(section.id);
								}}
								class={`flex items-center hover:bg-surface/50 px-3 border border-transparent rounded w-full min-w-0 h-9 text-base text-left cursor-pointer focus-visible:outline-2 focus-visible:outline-accent ${activeSection === section.id ? 'bg-accent/10 text-accent' : 'bg-background text-foreground'}`}
							>
								<span class="truncate">{section.name}</span>
							</button>
						</li>
					{/each}
				</ul>
			</OverlayScrollbarsComponent>
		</nav>
	{/snippet}
	<MainContent onScroll={updateActiveSection}>
		<div class="mx-auto px-6 py-12 w-full max-w-2xl settings-content page-width">
			<div class="w-full min-w-0">
				{#each sections as { name, id, component: Component }, i (id)}
					<section aria-labelledby={id}>
						<h1 class="pb-4 font-bold text-xl scroll-mt-16" {id}>{name}</h1>
						{#if id === 'about'}
							<MetaInformation
								onVersionTripleClick={() => setDebugVisibility(!config.show_debug_settings)}
							/>
						{:else}
							<Component />
						{/if}
					</section>
					{#if i < sections.length - 1}
						<Separator.Root class="bg-surface my-16 h-px" />
					{/if}
				{/each}
			</div>
		</div>
	</MainContent>
</ResizableNavigationLayout>

<style>
	.settings-content :global([data-setting-row]) {
		border: 1px solid var(--theme-elevated);
		padding: 1.25rem 1rem;
		/* Share adjoining borders, including rows inside component wrappers. */
		margin-bottom: -1px;
	}
</style>
