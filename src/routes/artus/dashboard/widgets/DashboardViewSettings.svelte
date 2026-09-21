<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Dialog } from 'bits-ui';
	import { mode } from 'mode-watcher';
	import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
	import Button from '$lib/components/Button.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
	import { dashboardViews, type DashboardView } from '../dashboard-views';

	let saveError = $state<string | null>(null);
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

	function toggleFavorite(value: DashboardView) {
		const values = orderedFavorites.map((view) => view.value);
		saveFavorites(
			favorites.has(value) ? values.filter((favorite) => favorite !== value) : [...values, value],
		);
	}

	function moveFavorite(value: DashboardView, offset: -1 | 1) {
		const values = orderedFavorites.map((view) => view.value);
		const from = values.indexOf(value);
		const to = from + offset;
		if (from < 0 || to < 0 || to >= values.length) return;
		[values[from], values[to]] = [values[to], values[from]];
		saveFavorites(values);
	}
</script>

<Dialog.Root>
	<Dialog.Trigger>
		<Button class="flex justify-center items-center p-2" aria-label="Configure dashboard views" title="Configure dashboard views">
			<Icon icon="material-symbols:settings-outline-rounded" class="size-4" />
		</Button>
	</Dialog.Trigger>
	<Dialog.Portal>
		<Dialog.Overlay class="data-[state=open]:animate-in data-[state=closed]:animate-out fixed inset-0 z-50 bg-black/50 data-[state=open]:backdrop-blur-xs" />
		<Dialog.Content class="fixed top-1/2 left-1/2 z-50 flex flex-col gap-4 bg-background p-6 border outline-hidden w-[min(42rem,calc(100vw-2rem))] h-[min(42rem,calc(100vh-2rem))] -translate-x-1/2 -translate-y-1/2">
			<div>
				<Dialog.Title class="font-bold text-lg font-expanded">Favorite dashboard views</Dialog.Title>
				<Dialog.Description class="mt-1 text-muted-foreground text-sm">
					Choose as many favorites as you like and arrange them in dashboard order.
				</Dialog.Description>
			</div>
			<OverlayScrollbarsComponent
				defer
				class="flex-1 min-h-0"
				options={{ scrollbars: { theme: scrollbarTheme, autoHide: 'move' } }}
			>
				<div class="flex flex-col gap-5 pr-3">
					<section aria-labelledby="favorite-views-heading">
						<div class="flex justify-between items-baseline mb-2">
							<h3 id="favorite-views-heading" class="font-medium text-sm">Favorites</h3>
							<span class="text-muted-foreground text-xs">{orderedFavorites.length} selected</span>
						</div>
						<div class="flex flex-col gap-2">
							{#each orderedFavorites as view, index (view.value)}
								<div class="flex bg-accent/10 border border-accent text-accent">
									<button
										type="button"
										onclick={() => toggleFavorite(view.value)}
										aria-label={`Remove ${view.label} from favorites`}
										class="flex flex-1 items-center gap-2 px-3 py-2 text-left cursor-pointer"
									>
										<Icon icon="material-symbols:star-rounded" class="size-4 shrink-0" />
										<span class="text-sm">{view.label}</span>
									</button>
									<div class="flex border-accent border-l">
										<button
											type="button"
											onclick={() => moveFavorite(view.value, -1)}
											disabled={index === 0}
											aria-label={`Move ${view.label} up`}
											class="hover:bg-accent/10 p-2 cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
										>
											<Icon icon="material-symbols:arrow-upward-rounded" class="size-4" />
										</button>
										<button
											type="button"
											onclick={() => moveFavorite(view.value, 1)}
											disabled={index === orderedFavorites.length - 1}
											aria-label={`Move ${view.label} down`}
											class="hover:bg-accent/10 p-2 cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
										>
											<Icon icon="material-symbols:arrow-downward-rounded" class="size-4" />
										</button>
									</div>
								</div>
							{:else}
								<p class="p-3 border text-muted-foreground text-sm">No favorite views selected.</p>
							{/each}
						</div>
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
			<div class="flex justify-end">
				<Dialog.Close><Button>Done</Button></Dialog.Close>
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
