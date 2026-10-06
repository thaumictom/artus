<script lang="ts">
	import { onMount } from 'svelte';
	import { appThemes } from '$lib/app-themes';
	import { config, loadSettings, watchAppTheme } from '$lib/settings.svelte';
	import { Toaster } from 'svelte-sonner';
	import 'overlayscrollbars/overlayscrollbars.css';
	import '../app.css';
	import type { Snippet } from 'svelte';
	import TooltipProvider from '$lib/components/TooltipProvider.svelte';

	let { children }: { children?: Snippet } = $props();
	let themeReady = $state(false);
	onMount(() => {
		void loadSettings()
			.then(() => { themeReady = true; })
			.catch((error) => console.error('Could not load theme:', error));
		const unwatch = watchAppTheme();
		return () => { void unwatch.then((stop) => stop()); };
	});
	$effect(() => {
		if (!themeReady) return;
		const root = document.documentElement;
		root.classList.remove(...appThemes.filter(({ value }) => value !== 'default').map(({ value }) => `theme-${value}`));
		if (config.app_theme !== 'default') root.classList.add(`theme-${config.app_theme}`);
		root.style.colorScheme = 'dark';
		if (!root.classList.contains('artus-document') || root.classList.contains('artus-theme-ready')) return;
		let revealFrame = 0;
		const frame = requestAnimationFrame(() => {
			// Give the fixed startup color one painted frame before the theme takes over.
			revealFrame = requestAnimationFrame(() => root.classList.add('artus-theme-ready'));
		});
		return () => { cancelAnimationFrame(frame); cancelAnimationFrame(revealFrame); };
	});
</script>

<Toaster position="bottom-center" theme="dark" />
<TooltipProvider>
	{@render children?.()}
</TooltipProvider>
