<script lang="ts">
	import { tick } from 'svelte';
	import Icon from '@iconify/svelte';
	import { mode } from 'mode-watcher';
	import {
		OverlayScrollbarsComponent,
		type OverlayScrollbarsComponentRef,
	} from 'overlayscrollbars-svelte';
	import { SortableList, sortItems } from '@rodrigodagostino/svelte-sortable-list';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
	import { dashboardViews, type DashboardView } from '../dashboard-views';

	const DRAG_TRANSITION_MS = 200;
	let saveError = $state<string | null>(null);
	let scrollbars = $state<OverlayScrollbarsComponentRef | null>(null);
	let favoriteValues = $derived(
		Array.isArray(config.dashboard_view_favorites) ? config.dashboard_view_favorites : [],
	);
	let orderedFavorites = $derived(
		[...dashboardViews]
			.filter((view) => favoriteValues.includes(view.value))
			.sort((a, b) => favoriteValues.indexOf(a.value) - favoriteValues.indexOf(b.value)),
	);
	let favorites = $derived(new Set(orderedFavorites.map((view) => view.value)));
	let availableViews = $derived(dashboardViews.filter((view) => !favorites.has(view.value)));
	let scrollbarTheme = $derived(mode.current === 'light' ? 'os-theme-dark' : 'os-theme-light');
	let saveQueue = Promise.resolve();

	function saveFavorites(values: DashboardView[]) {
		config.dashboard_view_favorites = values;
		saveError = null;
		saveQueue = saveQueue
			.catch(() => undefined)
			.then(() => updateSetting('dashboard_view_favorites'))
			.catch((error) => {
				console.error('Could not save dashboard favorites:', error);
				saveError = 'Could not save favorites.';
			});
	}

	async function toggleFavorite(value: DashboardView) {
		const isAdding = !favorites.has(value);
		const scrollElement = isAdding
			? scrollbars?.osInstance()?.elements().scrollOffsetElement
			: undefined;
		const scrollTop = scrollElement?.scrollTop;
		const values = orderedFavorites.map((view) => view.value);
		saveFavorites(
			favorites.has(value) ? values.filter((favorite) => favorite !== value) : [...values, value],
		);

		if (scrollTop === undefined) return;
		await tick();
		const updatedScrollElement = scrollbars?.osInstance()?.elements().scrollOffsetElement;
		if (updatedScrollElement) updatedScrollElement.scrollTop = scrollTop;
	}

	function noTransition() {
		return { duration: 0 };
	}

	function handleDragEnd(event: SortableList.RootEvents['ondragend']) {
		const { draggedItemIndex, targetItemIndex, isCanceled } = event;
		if (isCanceled || targetItemIndex === null || draggedItemIndex === targetItemIndex) return;

		const values = orderedFavorites.map((view) => view.value);
		saveFavorites(sortItems(values, draggedItemIndex, targetItemIndex));
	}
</script>

{#snippet trigger()}
	<Button
		class="flex justify-center items-center p-2"
		aria-label="Configure dashboard views"
		title="Configure dashboard views"
	>
		<Icon icon="material-symbols:settings-outline-rounded" class="size-4" />
	</Button>
{/snippet}

{#snippet title()}Favorite dashboard views{/snippet}

{#snippet description()}
	Choose as many favorites as you like and arrange them in dashboard order.
{/snippet}

{#snippet dialogClose()}<Button>Done</Button>{/snippet}

<Dialog {trigger} {title} {description} {dialogClose}>
	<OverlayScrollbarsComponent
		bind:this={scrollbars}
		defer
		class="flex-1 mr-1.75 min-w-0 min-h-0"
		options={{ scrollbars: { theme: scrollbarTheme, autoHide: 'move' } }}
	>
		<div class="flex flex-col gap-5 pr-4.25 pl-6">
			<section aria-labelledby="favorite-views-heading">
				<div class="flex justify-between items-baseline mb-2">
					<h3 id="favorite-views-heading" class="font-medium text-sm">Favorites</h3>
					<span class="text-muted-foreground text-xs">{orderedFavorites.length} selected</span>
				</div>
				<SortableList.Root
					gap={orderedFavorites.length === 0 ? 0 : 8}
					hasLockedAxis
					transition={{ duration: DRAG_TRANSITION_MS }}
					aria-labelledby="favorite-views-heading"
					ondragend={handleDragEnd}
					class="w-full dashboard-favorites-sortable"
				>
					{#each orderedFavorites as view, index (view.value)}
						<SortableList.Item
							id={`dashboard-favorite-${view.value}`}
							{index}
							aria-label={view.label}
							class="w-full"
							transitionIn={noTransition}
							transitionOut={noTransition}
						>
							<div
								class="flex items-stretch bg-background border border-accent w-full text-accent"
							>
								<span class="flex items-center px-3 text-muted-foreground">
									<Icon icon="material-symbols:drag-indicator-rounded" class="size-5" />
								</span>
								<span class="flex flex-1 items-center py-2 text-sm favorite-label">
									{view.label}
								</span>
								<SortableList.ItemRemove
									type="button"
									onclick={() => toggleFavorite(view.value)}
									aria-label={`Remove ${view.label} from favorites`}
									class="flex items-center hover:bg-accent/10 px-3 border-accent border-l cursor-pointer"
								>
									<Icon icon="material-symbols:close-rounded" class="size-4" />
								</SortableList.ItemRemove>
							</div>
						</SortableList.Item>
					{:else}
						<p class="p-3 border text-muted-foreground text-sm">No favorite views selected.</p>
					{/each}
				</SortableList.Root>
			</section>

			<section aria-labelledby="available-views-heading">
				<h3 id="available-views-heading" class="mb-2 font-medium text-sm">Available views</h3>
				<div class="gap-2 grid grid-cols-1 sm:grid-cols-2">
					{#each availableViews as view (view.value)}
						<button
							type="button"
							onclick={() => toggleFavorite(view.value)}
							aria-label={`Add ${view.label} to favorites`}
							class="flex items-center gap-2 hover:bg-surface px-3 py-2 border text-muted-foreground hover:text-foreground text-left cursor-pointer"
						>
							<Icon icon="material-symbols:star-outline-rounded" class="size-4 shrink-0" />
							<span class="text-sm">{view.label}</span>
						</button>
					{/each}
				</div>
			</section>
		</div>
	</OverlayScrollbarsComponent>
	{#if saveError}<p role="alert" class="text-danger text-sm">{saveError}</p>{/if}
</Dialog>

<style>
	/* Keep the floating drag copy unchanged and reduce its original slot to an outline. */
	:global(
			.dashboard-favorites-sortable
				.ssl-item[data-is-ghost='false'][data-drag-state*='ptr-drag']
				> div
		) {
		background-color: transparent;
		border-color: var(--color-muted-foreground);
	}

	:global(
			.dashboard-favorites-sortable
				.ssl-item[data-is-ghost='false'][data-drag-state*='ptr-drag']
				> div
				> *
		) {
		visibility: hidden;
	}
</style>
