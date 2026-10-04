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
	import { dashboardViewGroups, dashboardViews, type DashboardView } from '../dashboard-views';

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
		const selected = activeView;
		// A pinned shortcut selects content without revealing its group.
		const selectedFromPin = pinnedSelection === selected;
		pinnedSelection = null;
		if (selectedFromPin) return;
		const group = dashboardViewGroups.find((group) =>
			group.views.some((view) => view.value === selected),
		);
		// Reveal externally selected views without reopening groups the user collapses.
		if (group) untrack(() => (openedGroups[group.label] = true));
	});

	function selectView(value: DashboardView, fromPinned = false) {
		pinnedSelection = fromPinned ? value : null;
		const group = dashboardViewGroups.find((group) =>
			group.views.some((view) => view.value === value),
		);
		if (group && !fromPinned) openedGroups[group.label] = true;
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
	{#if count !== undefined}
		<span
			class="ml-1 px-2 border-2 rounded-full tabular-nums text-muted-foreground text-sm shrink-0"
		>
			{count}
		</span>
	{/if}
	{#if status !== undefined}
		<span class="ml-1 px-2 border-2 rounded-full tabular-nums text-accent text-sm shrink-0">
			{status}
		</span>
	{/if}
{/snippet}

{#snippet viewLink(view: (typeof dashboardViews)[number], fromPinned = false)}
	<div
		class={{
			'group/view flex flex-1 items-stretch pr-4 min-w-0 w-full text-base': true,
			'text-accent bg-accent/10': activeView === view.value,
			'text-foreground hover:bg-surface/50': activeView !== view.value,
		}}
	>
		<button
			type="button"
			aria-current={activeView === view.value ? 'page' : undefined}
			title={view.label}
			onclick={() => selectView(view.value, fromPinned)}
			class="flex flex-1 items-center gap-1 py-1.5 pr-2 pl-4 focus-visible:outline-2 focus-visible:outline-accent min-w-0 text-left cursor-pointer"
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
				class="flex justify-center items-center hover:bg-surface focus-visible:outline-2 focus-visible:outline-accent w-8 text-muted-foreground cursor-pointer shrink-0"
				class:text-accent={pins.has(view.value)}
			>
				<Icon icon={pins.has(view.value) ? 'lucide:pin-off' : 'lucide:pin'} class="size-4" />
			</button>
		{/if}
	</div>
{/snippet}

<nav
	aria-label="World state views"
	class="flex flex-col pr-0.5 w-56 h-full min-h-0 world-state-navigation shrink-0"
>
	<OverlayScrollbarsComponent
		bind:this={scrollbars}
		defer
		options={{ scrollbars: { theme: scrollbarTheme, autoHide: 'move' } }}
		class="flex-1 bg-background rounded-t-md w-full min-h-0"
	>
		<div class="flex flex-col">
			<section aria-labelledby="pinned-views-heading" class="mb-2 pb-2 border-surface border-b-2">
				<div class="flex justify-between items-center gap-2 p-4">
					<h2
						id="pinned-views-heading"
						class="font-medium text-muted-foreground text-sm uppercase tracking-wider"
					>
						Pinned
					</h2>
					<Button
						variant="link"
						size="none"
						disabled={!pinsReady}
						aria-label={editing ? 'Finish editing pins' : 'Edit pinned views'}
						aria-pressed={editing}
						onclick={() => (editing = !editing)}
						class="text-white text-sm"
					>
						{editing ? 'Done' : 'Edit'}
					</Button>
				</div>
				{#if editing}
					<p class="px-4 pb-3 text-muted-foreground text-sm">
						Drag to reorder. Pin views from the navigation below.
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
									class="w-full min-w-0"
									transitionIn={noTransition}
									transitionOut={noTransition}
								>
									<div
										class={{
											'flex items-center gap-1 hover:bg-surface/50 px-1 border border-transparent rounded w-full min-w-0 h-9': true,
											'bg-accent/10 text-accent': activeView === view.value,
											'bg-background text-foreground': activeView !== view.value,
										}}
									>
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
					<ul>
						{#each pinnedViews as view (view.value)}
							<li>{@render viewLink(view, true)}</li>
						{/each}
					</ul>
				{/if}
				{#if pinnedViews.length === 0}
					<p class="px-4 py-2.5 text-muted-foreground text-sm">
						{editing ? 'No pinned views yet.' : 'Use Edit to pin your favorite views.'}
					</p>
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

			{#each dashboardViewGroups as group (group.label)}
				<section aria-label={group.label} class="last:mb-2">
					<Collapsible
						bind:open={openedGroups[group.label]}
						triggerClass="flex justify-between items-center gap-2 hover:bg-elevated/50 px-4 py-3 w-full text-muted-foreground hover:text-foreground text-left cursor-pointer focus-visible:outline-2 focus-visible:outline-accent"
					>
						{#snippet button(open)}
							<span
								title={group.label}
								class="min-w-0 font-medium text-sm truncate uppercase tracking-wider"
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
								<ul class="pb-4" transition:slide={{ duration: 180 }}>
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
	<div class="bg-background p-4 border-surface border-t-2 shrink-0">
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

	:global(.world-state-pins-sortable .ssl-item[data-is-ghost='true'] > div),
	:global(.world-state-pins-sortable .ssl-item[data-drag-state*='kbd-drag'] > div) {
		background-color: var(--color-surface);
		border-color: var(--color-accent);
		box-shadow: 0 4px 12px rgb(0 0 0 / 20%);
	}
</style>
