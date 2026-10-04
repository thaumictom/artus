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
	onMount(() => {
		void loadSettings().catch((error) => console.error('Could not load theme:', error));
		const unwatch = watchAppTheme();
		return () => { void unwatch.then((stop) => stop()); };
	});
	$effect(() => {
		const root = document.documentElement;
		root.classList.remove(...appThemes.filter(({ value }) => value !== 'default').map(({ value }) => `theme-${value}`));
		if (config.app_theme !== 'default') root.classList.add(`theme-${config.app_theme}`);
		root.style.colorScheme = 'dark';
	});
</script>

<Toaster position="bottom-center" theme="dark" />
<TooltipProvider>
	{@render children?.()}
</TooltipProvider>
