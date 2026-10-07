<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import { Pane, PaneGroup, PaneResizer } from 'paneforge';
	import { config, loadSettings, updateSetting } from '$lib/settings.svelte';

	let {
		navigation,
		children,
		resizeLabel,
	}: {
		navigation: Snippet;
		children: Snippet;
		resizeLabel: string;
	} = $props();

	const RESIZER_WIDTH = 6;
	let containerWidth = $state(0);
	let initialNavigationWidth = $state(240);
	let ready = $state(false);
	let widthSaveTimer: ReturnType<typeof setTimeout> | undefined;
	// PaneForge divides the space left after the resizer using percentages.
	let paneWidth = $derived(Math.max(1, containerWidth - RESIZER_WIDTH));
	let navigationMinSize = $derived(Math.min(100, (200 / paneWidth) * 100));
	let navigationMaxSize = $derived(Math.min(100, (350 / paneWidth) * 100));
	let navigationDefaultSize = $derived(Math.min(100, (initialNavigationWidth / paneWidth) * 100));

	onMount(() => {
		void loadSettings()
			.then(() => {
				initialNavigationWidth = config.dashboard_navigation_width;
				ready = true;
			})
			.catch((error) => {
				console.error('Could not load navigation width:', error);
				ready = true;
			});
		return () => saveNavigationWidth();
	});

	function saveNavigationWidth() {
		if (widthSaveTimer === undefined) return;
		clearTimeout(widthSaveTimer);
		widthSaveTimer = undefined;
		void updateSetting('dashboard_navigation_width').catch((error) => {
			console.error('Could not save navigation width:', error);
		});
	}

	function onNavigationResize(size: number) {
		// Store pixels so restoring the width does not depend on the window size.
		const width = Math.round(size / 100 * paneWidth);
		if (width === config.dashboard_navigation_width) return;
		config.dashboard_navigation_width = width;
		clearTimeout(widthSaveTimer);
		widthSaveTimer = setTimeout(saveNavigationWidth, 250);
	}
</script>

<div
	class="flex flex-1 w-full min-w-0 h-full min-h-0 overflow-hidden"
	bind:clientWidth={containerWidth}
>
	{#if containerWidth > 0 && ready}
		<PaneGroup direction="horizontal" class="min-w-0 min-h-0">
			<Pane
				defaultSize={navigationDefaultSize}
				minSize={navigationMinSize}
				maxSize={navigationMaxSize}
				onResize={onNavigationResize}
			>
				{@render navigation()}
			</Pane>
			<PaneResizer
				aria-label={resizeLabel}
				onDraggingChange={(dragging) => { if (!dragging) saveNavigationWidth(); }}
				style={`width: ${RESIZER_WIDTH}px`}
				class="group flex justify-center items-center data-[active]:bg-surface hover:bg-surface rounded focus-visible:outline-2 focus-visible:outline-accent shrink-0"
			>
				<div class="bg-muted-foreground/30 group-data-[active]:bg-accent group-hover:bg-accent rounded w-0.5 h-8"></div>
			</PaneResizer>
			<Pane class="flex min-w-0 min-h-0">
				{@render children()}
			</Pane>
		</PaneGroup>
	{/if}
</div>
