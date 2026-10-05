<script lang="ts">
	import { onMount, tick, untrack, type Snippet } from 'svelte';
	import { slide } from 'svelte/transition';
	import Icon from '@iconify/svelte';
	import type { WorldState } from 'warframe-worldstate-parser';
	import {
		OverlayScrollbarsComponent,
		type OverlayScrollbarsComponentRef,
	} from 'overlayscrollbars-svelte';
	import { SortableList, sortItems } from '@rodrigodagostino/svelte-sortable-list';
	import Button from '$lib/components/Button.svelte';
	import Collapsible from '$lib/components/Collapsible.svelte';
	import { config, loadSettings, updateSetting } from '$lib/settings.svelte';
	import { initializeAlertCompletions } from '$lib/alert-completions.svelte';
	import { dashboardViewGroups, dashboardViews, isDashboardViewVisible, type DashboardView } from '../dashboard-views';

	let {
		activeView,
		world,
		now,
		onSelect,
		footer,
	}: {
		activeView: DashboardView;
		world: WorldState | null;
		now: number;
		onSelect: (view: DashboardView) => void;
		footer: Snippet;
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
	let pinnedSelection: DashboardView | null = null;
	let pinnedValues = $derived(config.dashboard_view_favorites);
	let visibleGroups = $derived(dashboardViewGroups.filter((group) =>
		!group.debugOnly || config.show_unused_dashboard_views,
	));
	let pinnedViews = $derived(
		[...dashboardViews]
			.filter((view) => pinnedValues.includes(view.value)
				&& isDashboardViewVisible(view.value, config.show_unused_dashboard_views))
			.sort((a, b) => pinnedValues.indexOf(a.value) - pinnedValues.indexOf(b.value)),
	);
	let pins = $derived(new Set(pinnedViews.map((view) => view.value)));

	onMount(() => {
		void initializeAlertCompletions().catch((error) => {
			console.error('Could not load alert completions for navigation:', error);
		});
		void loadSettings()
			.then(() => (pinsReady = true))
			.catch((error) => {
				console.error('Could not load world state pins:', error);
				loadError = 'Could not load pins.';
			});
	});

	$effect(() => {
		const selected = activeView;
		// A pinned shortcut selects content without revealing its group.
		const selectedFromPin = pinnedSelection === selected;
		pinnedSelection = null;
		if (selectedFromPin) return;
		const group = visibleGroups.find((group) =>
			group.views.some((view) => view.value === selected),
		);
		// Reveal externally selected views without reopening groups the user collapses.
		if (group) untrack(() => (openedGroups[group.label] = true));
	});

	function selectView(value: DashboardView, fromPinned = false) {
		pinnedSelection = fromPinned ? value : null;
		const group = visibleGroups.find((group) =>
			group.views.some((view) => view.value === value),
		);
		if (group && !fromPinned) openedGroups[group.label] = true;
		onSelect(value);
	}

	function savePins(values: DashboardView[]) {
		// Editing visible pins must preserve pins hidden by the Debug toggle.
		const hiddenPins = config.dashboard_view_favorites.filter((value) =>
			!isDashboardViewVisible(value, config.show_unused_dashboard_views),
		);
		config.dashboard_view_favorites = [...values, ...hiddenPins];
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

	function viewRowClass(value: DashboardView) {
		return `flex items-center gap-1 hover:bg-surface/50 border border-transparent rounded w-full min-w-0 h-9 text-base ${
			activeView === value ? 'bg-accent/10 text-accent' : 'bg-background text-foreground'
		}`;
	}

	function handleDragEnd(event: SortableList.RootEvents['ondragend']) {
		const { draggedItemIndex, targetItemIndex, isCanceled } = event;
		if (isCanceled || targetItemIndex === null || draggedItemIndex === targetItemIndex) return;
		savePins(
			sortItems(
				pinnedViews.map((view) => view.value),
				draggedItemIndex,
				targetItemIndex,
			),
		);
	}
</script>

{#snippet viewLabel(view: (typeof dashboardViews)[number])}
	{@const count = world && 'itemCount' in view ? view.itemCount(world, now) : undefined}
	{@const status = world && 'statusLabel' in view ? view.statusLabel(world, now) : undefined}
	<span class="min-w-0 truncate">{view.label}</span>
	{#if count !== undefined && count > 0 && !editing}
		<span
			class="flex justify-center items-center bg-surface ml-1 px-1.25 border rounded-full tabular-nums text-muted-foreground text-trim text-xs shrink-0"
		>
			{count}
		</span>
	{/if}
	{#if status !== undefined && !editing}
		<span
			class="bg-surface ml-1 px-1.5 border rounded-full tabular-nums text-accent text-xs shrink-0"
		>
			{status}
		</span>
	{/if}
{/snippet}

{#snippet viewLink(view: (typeof dashboardViews)[number], fromPinned = false)}
	<div class="relative w-full min-w-0">
		<button
			type="button"
			aria-current={activeView === view.value ? 'page' : undefined}
			title={view.label}
			onclick={() => selectView(view.value, fromPinned)}
			class={`${viewRowClass(view.value)} pl-3 ${editing && !fromPinned ? 'pr-10' : 'pr-3'} focus-visible:outline-2 focus-visible:outline-accent text-left cursor-pointer`}
		>
			{@render viewLabel(view)}
		</button>
		{#if editing && !fromPinned}
			<button
				type="button"
				aria-label={pins.has(view.value) ? `Unpin ${view.label}` : `Pin ${view.label}`}
				title={pins.has(view.value) ? `Unpin ${view.label}` : `Pin ${view.label}`}
				aria-pressed={pins.has(view.value)}
				onclick={() => togglePin(view.value)}
				class="absolute right-1 top-1/2 -translate-y-1/2 flex justify-center items-center hover:bg-surface rounded focus-visible:outline-2 focus-visible:outline-accent w-7 h-7 text-muted-foreground hover:text-foreground cursor-pointer"
				class:text-accent={pins.has(view.value)}
			>
				<Icon icon={pins.has(view.value) ? 'lucide:pin-off' : 'lucide:pin'} class="size-4" />
			</button>
		{/if}
	</div>
{/snippet}

<nav
	aria-label="World state views"
	class="flex flex-col bg-background rounded-t-md w-full h-full min-h-0 world-state-navigation"
>
	<OverlayScrollbarsComponent
		bind:this={scrollbars}
		defer
		options={{ scrollbars: { theme: scrollbarTheme, autoHide: 'move' } }}
		class="flex-1 w-full min-h-0"
	>
		<div class="flex flex-col">
			<section aria-labelledby="pinned-views-heading">
				<div class="group/pinned-header flex justify-between items-center gap-2 p-3 pt-4">
					<h2
						id="pinned-views-heading"
						class="px-3 font-semibold text-muted-foreground text-xs uppercase tracking-widest"
					>
						Pinned
					</h2>
					<span
						class={editing
							? ''
							: 'opacity-0 pointer-events-none group-hover/pinned-header:opacity-100 group-hover/pinned-header:pointer-events-auto group-focus-within/pinned-header:opacity-100 group-focus-within/pinned-header:pointer-events-auto'}
					>
						<Button
							variant="link"
							size="none"
							disabled={!pinsReady}
							aria-label={editing ? 'Finish editing pins' : 'Edit pinned views'}
							aria-pressed={editing}
							onclick={() => (editing = !editing)}
							class="flex items-center gap-1 mr-3 text-sm"
						>
							{#if editing}
								<Icon icon="material-symbols:check" class="size-4" />
								done
							{:else}
								<Icon icon="material-symbols:edit" class="size-4" />
								edit
							{/if}
						</Button>
					</span>
				</div>
				{#if editing}
					<p class="px-6 pb-3 text-muted-foreground text-xs">
						Pin items from the navigation below. Drag pinned items to reorder.
					</p>
					<div class="px-3 min-w-0">
						<SortableList.Root
							gap={pinnedViews.length === 0 ? 0 : 2}
							hasLockedAxis
							transition={{ duration: 200 }}
							aria-labelledby="pinned-views-heading"
							ondragend={handleDragEnd}
							class="w-full min-w-0 world-state-pins-sortable"
						>
							{#each pinnedViews as view, index (view.value)}
								<SortableList.Item
									id={`world-state-pin-${view.value}`}
									{index}
									aria-label={view.label}
									class="w-full min-w-0 world-state-pin-sortable-item"
									transitionIn={noTransition}
									transitionOut={noTransition}
								>
									<div class={`${viewRowClass(view.value)} px-1`}>
										<SortableList.ItemHandle
											class="flex justify-center items-center hover:bg-surface rounded focus-visible:outline-2 focus-visible:outline-accent w-6 h-7 text-muted-foreground cursor-grab active:cursor-grabbing shrink-0"
										>
											<Icon icon="material-symbols:drag-indicator-rounded" class="size-4" />
										</SortableList.ItemHandle>
										<span
											title={view.label}
											class="flex flex-1 items-center gap-1 min-w-0 text-base"
										>
											{@render viewLabel(view)}
										</span>
										<SortableList.ItemRemove
											type="button"
											onclick={() => togglePin(view.value)}
											aria-label={`Unpin ${view.label}`}
											title={`Unpin ${view.label}`}
											class="flex justify-center items-center hover:bg-surface rounded focus-visible:outline-2 focus-visible:outline-accent w-7 h-7 text-muted-foreground hover:text-foreground cursor-pointer shrink-0"
										>
											<Icon icon="material-symbols:close-rounded" class="size-4" />
										</SortableList.ItemRemove>
									</div>
								</SortableList.Item>
							{/each}
						</SortableList.Root>
					</div>
				{:else}
					<ul class="flex flex-col gap-0.5 px-3">
						{#each pinnedViews as view (view.value)}
							<li>{@render viewLink(view, true)}</li>
						{/each}
					</ul>
				{/if}
				{#if pinnedViews.length === 0}
					<p class="px-6 text-muted-foreground text-sm">No pinned views.</p>
				{/if}
				{#if loadError}<p role="alert" class="px-6 py-2.5 text-danger text-sm">{loadError}</p>{/if}
				{#if saveError}
					<div role="alert" class="flex items-center gap-2 px-6 py-2.5 text-danger text-sm">
						<span>{saveError}</span>
						<Button
							variant="link"
							size="none"
							onclick={() => savePins(pinnedViews.map((view) => view.value))}
						>
							Retry
						</Button>
					</div>
				{/if}
			</section>
			<div class="bg-surface m-3 h-px"></div>
			{#each visibleGroups as group (group.label)}
				<section aria-label={group.label} class="mb-3 px-3">
					<Collapsible
						bind:open={openedGroups[group.label]}
						triggerClass="flex justify-between items-center gap-1 bg-background hover:bg-surface/50 px-3 mb-0.5 border border-transparent rounded w-full min-w-0 h-9 text-muted-foreground hover:text-foreground text-base text-left cursor-pointer focus-visible:outline-2 focus-visible:outline-accent uppercase text-xs tracking-widest"
					>
						{#snippet button(open)}
							<span
								title={group.label}
								class="min-w-0 font-semibold truncate"
								class:text-accent={group.views.some((view) => view.value === activeView)}
							>
								{group.label}
							</span>
							<Icon
								icon="material-symbols:keyboard-arrow-down-rounded"
								class={`size-4 shrink-0 transition-transform ${open ? 'rotate-180' : ''}`}
							/>
						{/snippet}
						{#snippet content(open)}
							{#if open}
								<ul class="flex flex-col gap-0.5 pb-2" transition:slide={{ duration: 180 }}>
									{#each group.views as view (view.value)}
										<li class="flex items-stretch">
											{@render viewLink(view)}
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
	<div class="bg-surface mx-3 mb-4 h-px"></div>
	<div class="shrink-0">
		{@render footer()}
	</div>
</nav>

<style>
	.world-state-navigation :global([data-overlayscrollbars-viewport]) {
		overscroll-behavior: contain;
	}

	/* Keep the dragged copy visible and leave an outline at its original position. */
	:global(
			.world-state-pins-sortable .ssl-item[data-is-ghost='false'][data-drag-state*='ptr-drag'] > div
		) {
		background-color: transparent;
		border-color: var(--color-muted-foreground);
		border-style: dashed;
	}

	:global(
			.world-state-pins-sortable
				.ssl-item[data-is-ghost='false'][data-drag-state*='ptr-drag']
				> div
				> *
		) {
		visibility: hidden;
	}

	/* The library portals the dragged copy outside the list, retaining the item's class. */
	:global(.world-state-pin-sortable-item[data-is-ghost='true'] > div),
	:global(.world-state-pin-sortable-item[data-drag-state*='kbd-drag'] > div) {
		background-color: color-mix(in oklab, var(--color-surface) 60%, transparent);
		border-color: var(--color-accent);
		box-shadow: 0 4px 12px rgb(0 0 0 / 20%);
	}
</style>
