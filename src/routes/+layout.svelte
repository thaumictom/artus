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
		if (root.dataset.startupTheme) {
			root.classList.remove(`theme-${root.dataset.startupTheme}`);
			delete root.dataset.startupTheme;
		}
		root.classList.remove(...appThemes.filter(({ value }) => value !== 'default').map(({ value }) => `theme-${value}`));
		if (config.app_theme !== 'default') root.classList.add(`theme-${config.app_theme}`);
		root.style.colorScheme = 'dark';
		if (!root.classList.contains('artus-document')) return;
		const frame = requestAnimationFrame(() => {
			const background = getComputedStyle(document.body).backgroundColor;
			try {
				localStorage.setItem('artus-startup-theme', JSON.stringify({ theme: config.app_theme, background }));
				root.style.setProperty('--artus-startup-background', background);
			} catch { /* The startup fallback still works if storage is unavailable. */ }
		});
		return () => cancelAnimationFrame(frame);
	});
</script>

<Toaster position="bottom-center" theme="dark" />
<TooltipProvider>
	{@render children?.()}
</TooltipProvider>
