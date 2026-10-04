<script lang="ts">
	import { onMount, tick, untrack } from 'svelte';
	import { slide } from 'svelte/transition';
	import Icon from '@iconify/svelte';
	import {
		OverlayScrollbarsComponent,
		type OverlayScrollbarsComponentRef,
	} from 'overlayscrollbars-svelte';
	import { SortableList, sortItems } from '@rodrigodagostino/svelte-sortable-list';
	import Button from '$lib/components/Button.svelte';
	import Collapsible from '$lib/components/Collapsible.svelte';
	import { config, loadSettings, updateSetting } from '$lib/settings.svelte';
	import { dashboardViewGroups, dashboardViews, type DashboardView } from '../dashboard-views';

	let {
		activeView,
		onSelect,
	}: {
		activeView: DashboardView;
		onSelect: (view: DashboardView) => void;
	} = $props();

	const scrollbarTheme = 'os-theme-light';
	let editing = $state(false);
	let pinsReady = $state(false);
	let loadError = $state<string | null>(null);
	let saveError = $state<string | null>(null);
	let scrollbars = $state<OverlayScrollbarsComponentRef | null>(null);
	let openedGroups = $state<Record<string, boolean>>(
		Object.fromEntries(dashboardViewGroups.map((group) => [group.label, false])),
	);
	let saveQueue = Promise.resolve();
	let pinnedValues = $derived(config.dashboard_view_favorites);
	let pinnedViews = $derived(
		[...dashboardViews]
			.filter((view) => pinnedValues.includes(view.value))
			.sort((a, b) => pinnedValues.indexOf(a.value) - pinnedValues.indexOf(b.value)),
	);
	let pins = $derived(new Set(pinnedViews.map((view) => view.value)));

	onMount(() => {
		void loadSettings()
			.then(() => (pinsReady = true))
			.catch((error) => {
				console.error('Could not load world state pins:', error);
				loadError = 'Could not load pins.';
			});
	});

	$effect(() => {
		const group = dashboardViewGroups.find((group) =>
			group.views.some((view) => view.value === activeView),
		);
		// Reveal externally selected views without reopening groups the user collapses.
		if (group) untrack(() => (openedGroups[group.label] = true));
	});

	function selectView(value: DashboardView) {
		const group = dashboardViewGroups.find((group) =>
			group.views.some((view) => view.value === value),
		);
		if (group) openedGroups[group.label] = true;
		onSelect(value);
	}

	function savePins(values: DashboardView[]) {
		config.dashboard_view_favorites = values;
		saveError = null;
		saveQueue = saveQueue
			.catch(() => undefined)
			.then(() => updateSetting('dashboard_view_favorites'))
			.catch((error) => {
				console.error('Could not save world state pins:', error);
				saveError = 'Could not save pins.';
			});
	}

	async function togglePin(value: DashboardView) {
		const scrollElement = scrollbars?.osInstance()?.elements().scrollOffsetElement;
		const scrollTop = scrollElement?.scrollTop;
		const values = pinnedViews.map((view) => view.value);
		savePins(pins.has(value) ? values.filter((pin) => pin !== value) : [...values, value]);

		// Adding a row above the groups should not reset the navigation scroll position.
		await tick();
		if (scrollElement && scrollTop !== undefined) scrollElement.scrollTop = scrollTop;
	}

	function noTransition() {
		return { duration: 0 };
	}

	function handleDragEnd(event: SortableList.RootEvents['ondragend']) {
		const { draggedItemIndex, targetItemIndex, isCanceled } = event;
		if (isCanceled || targetItemIndex === null || draggedItemIndex === targetItemIndex) return;
		savePins(sortItems(pinnedViews.map((view) => view.value), draggedItemIndex, targetItemIndex));
	}
</script>

{#snippet viewLink(view: (typeof dashboardViews)[number])}
	<button
		type="button"
		aria-current={activeView === view.value ? 'page' : undefined}
		onclick={() => selectView(view.value)}
		class={{
			'block flex-1 px-3 py-1.5 border-l-2 min-w-0 w-full text-sm text-left cursor-pointer transition-colors focus-visible:outline-2 focus-visible:outline-accent focus-visible:outline-offset-[-2px]': true,
			'bg-accent/10 border-accent text-accent': activeView === view.value,
			'border-transparent text-muted-foreground hover:bg-surface hover:text-foreground': activeView !== view.value,
		}}
	>
		{view.label}
	</button>
{/snippet}

<nav
	aria-label="World state views"
	class="world-state-navigation top-0 sticky self-start bg-background border-r w-52 h-[calc(100vh-2.5rem)] shrink-0"
>
	<OverlayScrollbarsComponent
		bind:this={scrollbars}
		defer
		options={{ scrollbars: { theme: scrollbarTheme, autoHide: 'move' } }}
		class="w-full h-full"
	>
		<div class="space-y-3 px-2 py-4">
			<section aria-labelledby="pinned-views-heading" class="pb-3 border-b border-border-secondary">
				<div class="flex justify-between items-center gap-2 pl-3 pb-2">
					<h2 id="pinned-views-heading" class="font-medium text-muted-foreground text-xs uppercase tracking-wider">
						Pinned
					</h2>
					<Button
						variant="ghost"
						size="small"
						disabled={!pinsReady}
						aria-label={editing ? 'Finish editing pins' : 'Edit pinned views'}
						aria-pressed={editing}
						onclick={() => (editing = !editing)}
						class="text-xs"
					>
						{editing ? 'Done' : 'Edit'}
					</Button>
				</div>
				{#if editing}
					<p class="px-3 pb-3 text-muted-foreground text-xs">Drag to reorder. Pin views from the groups below.</p>
					<SortableList.Root
						gap={pinnedViews.length === 0 ? 0 : 4}
						hasLockedAxis
						transition={{ duration: 200 }}
						aria-labelledby="pinned-views-heading"
						ondragend={handleDragEnd}
						class="w-full world-state-pins-sortable"
					>
						{#each pinnedViews as view, index (view.value)}
							<SortableList.Item
								id={`world-state-pin-${view.value}`}
								{index}
								aria-label={view.label}
								class="w-full"
								transitionIn={noTransition}
								transitionOut={noTransition}
							>
								<div class="flex items-stretch bg-background border border-accent/40 w-full text-accent">
									<SortableList.ItemHandle class="flex items-center px-1.5 text-muted-foreground">
										<Icon icon="material-symbols:drag-indicator-rounded" class="size-4" />
									</SortableList.ItemHandle>
									<span class="flex-1 py-1.5 min-w-0 text-sm">{view.label}</span>
									<SortableList.ItemRemove
										type="button"
										onclick={() => togglePin(view.value)}
										aria-label={`Unpin ${view.label}`}
										title={`Unpin ${view.label}`}
										class="flex items-center hover:bg-accent/10 px-1.5 cursor-pointer focus-visible:outline-2 focus-visible:outline-accent"
									>
										<Icon icon="material-symbols:close-rounded" class="size-4" />
									</SortableList.ItemRemove>
								</div>
							</SortableList.Item>
						{/each}
					</SortableList.Root>
				{:else}
					<ul class="space-y-0.5">
						{#each pinnedViews as view (view.value)}
							<li>{@render viewLink(view)}</li>
						{/each}
					</ul>
				{/if}
				{#if pinnedViews.length === 0}
					<p class="px-3 py-1 text-muted-foreground text-xs">
						{editing ? 'No pinned views yet.' : 'Use Edit to pin your favorite views.'}
					</p>
				{/if}
				{#if loadError}<p role="alert" class="px-3 py-2 text-danger text-xs">{loadError}</p>{/if}
				{#if saveError}
					<div role="alert" class="flex items-center gap-2 px-3 py-2 text-danger text-xs">
						<span>{saveError}</span>
						<Button variant="link" size="none" onclick={() => savePins(pinnedViews.map((view) => view.value))}>Retry</Button>
					</div>
				{/if}
			</section>

			{#each dashboardViewGroups as group (group.label)}
				<section aria-label={group.label}>
					<Collapsible
						bind:open={openedGroups[group.label]}
						triggerClass="flex justify-between items-center gap-2 hover:bg-surface px-3 py-2 w-full text-muted-foreground hover:text-foreground text-left cursor-pointer focus-visible:outline-2 focus-visible:outline-accent"
					>
						{#snippet button(open)}
							<span
								class="font-medium text-xs uppercase tracking-wider"
								class:text-accent={group.views.some((view) => view.value === activeView)}
							>{group.label}</span>
							<Icon
								icon="material-symbols:keyboard-arrow-down-rounded"
								class={`size-4 shrink-0 transition-transform ${open ? 'rotate-180' : ''}`}
							/>
						{/snippet}
						{#snippet content(open)}
							{#if open}
								<ul class="space-y-0.5 pt-1" transition:slide={{ duration: 180 }}>
									{#each group.views as view (view.value)}
										<li class="flex items-stretch">
											{@render viewLink(view)}
											{#if editing}
												<button
													type="button"
													aria-label={pins.has(view.value) ? `Unpin ${view.label}` : `Pin ${view.label}`}
													title={pins.has(view.value) ? `Unpin ${view.label}` : `Pin ${view.label}`}
													aria-pressed={pins.has(view.value)}
													onclick={() => togglePin(view.value)}
													class="flex justify-center items-center hover:bg-surface w-8 text-muted-foreground shrink-0 cursor-pointer focus-visible:outline-2 focus-visible:outline-accent"
													class:text-accent={pins.has(view.value)}
												>
													<Icon icon={pins.has(view.value) ? 'lucide:pin-off' : 'lucide:pin'} class="size-4" />
												</button>
											{/if}
										</li>
									{/each}
								</ul>
							{/if}
						{/snippet}
					</Collapsible>
				</section>
			{/each}
		</div>
	</OverlayScrollbarsComponent>
</nav>

<style>
	.world-state-navigation :global([data-overlayscrollbars-viewport]) {
		overscroll-behavior: contain;
	}

	/* Keep the dragged copy visible and leave an outline at its original position. */
	:global(.world-state-pins-sortable .ssl-item[data-is-ghost='false'][data-drag-state*='ptr-drag'] > div) {
		background-color: transparent;
		border-color: var(--color-muted-foreground);
	}

	:global(.world-state-pins-sortable .ssl-item[data-is-ghost='false'][data-drag-state*='ptr-drag'] > div > *) {
		visibility: hidden;
	}
</style>
