<script lang="ts">
	import { onMount } from 'svelte';

	import Select from '$lib/components/Select.svelte';
	import type { OcrThemeOption } from '$lib/types';
	import { getOcrThemeSettings, setOcrTheme } from '../settings-api';

	type SelectItem = {
		value: string;
		label: string;
		disabled?: boolean;
	};

	let items = $state<SelectItem[]>([]);
	let selectedTheme = $state('');
	let lastAppliedTheme = $state<string | null>(null);
	let isLoading = $state(true);
	let isSaving = $state(false);
	let status = $state<string | null>(null);

	function formatThemeName(theme: string): string {
		return theme
			.toLowerCase()
			.split('_')
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(' ');
	}

	function mapThemeItems(themes: OcrThemeOption[]): SelectItem[] {
		return themes.map((theme) => ({
			value: theme.name,
			label: formatThemeName(theme.name),
		}));
	}

	async function loadThemes() {
		isLoading = true;
		status = null;

		try {
			const settings = await getOcrThemeSettings();
			items = mapThemeItems(settings.themes);
			selectedTheme = settings.selected_theme;
			lastAppliedTheme = settings.selected_theme;
		} catch (error) {
			status = String(error);
		} finally {
			isLoading = false;
		}
	}

	async function saveTheme(theme: string) {
		if (!theme || theme === lastAppliedTheme) {
			return;
		}

		isSaving = true;
		status = null;

		try {
			await setOcrTheme(theme);
			lastAppliedTheme = theme;
			status = 'Saved';
		} catch (error) {
			status = String(error);
		} finally {
			isSaving = false;
		}
	}

	function onThemeChange(nextTheme: string) {
		selectedTheme = nextTheme;
		if (isLoading) {
			return;
		}

		void saveTheme(nextTheme);
	}

	onMount(() => {
		void loadThemes();
	});
</script>

<p class="mb-1">In-Game Theme</p>
<Select
	type="single"
	{items}
	value={selectedTheme}
	onValueChange={onThemeChange}
	disabled={isLoading || isSaving}
/>

{#if status}
	<p class="mt-1 text-muted-foreground text-xs">{status}</p>
{/if}
