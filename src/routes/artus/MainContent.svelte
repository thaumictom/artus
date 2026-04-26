<script lang="ts">
	import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
	import type { Snippet } from 'svelte';
	import { mode } from 'mode-watcher';

	let scrollbarTheme = $derived.by(() =>
		mode.current === 'light' ? 'os-theme-dark' : 'os-theme-light',
	);

	let { children }: { children: Snippet } = $props();
</script>

<div class="relative flex-1">
	<div
		class="right-4 left-0 absolute inset-y-0 bg-background rounded-t-md pointer-events-none"
	></div>

	<div class="absolute inset-0">
		<OverlayScrollbarsComponent
			defer
			options={{ scrollbars: { theme: scrollbarTheme, autoHide: 'move' } }}
			class="w-full h-full"
		>
			<div class="pr-4 min-h-full">
				{@render children?.()}
			</div>
		</OverlayScrollbarsComponent>
	</div>
</div>
