<script lang="ts">
	import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
	import type { Snippet } from 'svelte';
	import { config } from '$lib/settings.svelte';

	const scrollbarTheme = 'os-theme-light';

	let { children }: { children: Snippet } = $props();
</script>

<div class="relative flex-1 min-w-0 min-h-0">
	<div
		class="right-4 left-0 absolute inset-y-0 bg-background rounded-t-md pointer-events-none"
	></div>

	<div class="absolute inset-0">
		<OverlayScrollbarsComponent
			defer
			options={{ scrollbars: { theme: scrollbarTheme, autoHide: 'move' } }}
			class="w-full h-full"
		>
			<div class="pr-4 min-h-full" class:full-width={config.full_width_content}>
				{@render children?.()}
			</div>
		</OverlayScrollbarsComponent>
	</div>
</div>

<style>
	/* Only page content opts in; controls, skeletons, and popups keep their own limits. */
	.full-width :global(.page-width) {
		max-width: none;
	}
</style>
