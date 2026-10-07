<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { customThemeColors } from '$lib/custom-theme';
	import { appThemes, accentColors, isSpecialtyTheme } from '$lib/app-themes';
	import { config, loadSettings, watchAppTheme } from '$lib/settings.svelte';
	import { Toaster } from 'svelte-sonner';
	import 'overlayscrollbars/overlayscrollbars.css';
	import '../app.css';
	import type { Snippet } from 'svelte';
	import TooltipProvider from '$lib/components/TooltipProvider.svelte';

	let { children }: { children?: Snippet } = $props();
	let themeReady = $state(false);
	let themeMode = $derived.by(() => {
		if (!config.app_theme_custom_enabled && isSpecialtyTheme(config.app_theme)) return 'dark';
		// Each window resolves its own appearance without changing the saved app mode.
		if (page.route.id === '/overlay' && config.overlay_opposite_theme) {
			return config.app_theme_mode === 'dark' ? 'light' : 'dark';
		}
		return config.app_theme_mode;
	});
	onMount(() => {
		function focusItemSearch(event: KeyboardEvent) {
			if (!event.ctrlKey || event.altKey || event.metaKey || event.shiftKey || event.key.toLowerCase() !== 'f') return;
			event.preventDefault();
			const dialog = document.querySelector('[role="dialog"][data-state="open"], [role="alertdialog"][data-state="open"]');
			const search = (dialog ?? document).querySelector<HTMLInputElement>('input[data-item-search]:not(:disabled)');
			search?.focus();
			search?.select();
		}
		window.addEventListener('keydown', focusItemSearch, true);
		void loadSettings()
			.then(() => { themeReady = true; })
			.catch((error) => console.error('Could not load theme:', error));
		const unwatch = watchAppTheme();
		return () => {
			window.removeEventListener('keydown', focusItemSearch, true);
			void unwatch.then((stop) => stop());
		};
	});
	$effect(() => {
		if (!themeReady) return;
		const root = document.documentElement;
		root.classList.remove(
			...appThemes.map(({ value }) => `theme-${value}`),
			...accentColors.map((color) => `theme-accent-${color}`),
		);
		root.classList.add(`theme-${config.app_theme}`, `theme-accent-${config.app_theme_accent}`);
		root.dataset.themeMode = themeMode;
		root.dataset.accentShade = config.app_theme_accent_shade;
		root.style.colorScheme = themeMode;
		for (const { value } of customThemeColors) {
			const color = config.app_theme_custom_enabled && config.app_theme_custom_colors?.[themeMode][value];
			if (color) root.style.setProperty(`--theme-${value}`, color);
			else root.style.removeProperty(`--theme-${value}`);
		}
	});
</script>

<Toaster position="bottom-center" theme={themeMode} />
<TooltipProvider>
	{@render children?.()}
</TooltipProvider>
