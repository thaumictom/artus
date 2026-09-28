<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { appThemes } from '$lib/app-themes';
	import { config, updateSetting } from '$lib/settings.svelte';
	import type { OcrThemeOption } from '$lib/types';
	import Select from '$lib/components/Select.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';

	let inGameThemes = $state<OcrThemeOption[]>([]);
	let selectItems = $derived(
		appThemes.map(({ value, label }) => {
			const matchingTheme = inGameThemes.find(
				({ name }) => name.toLowerCase().replaceAll('_', '-') === value,
			);
			return {
				value,
				label,
				swatchColors: matchingTheme
					? ([toCssRgb(matchingTheme.rgb), toCssRgb(matchingTheme.highlight_rgb)] as [
							string,
							string,
						])
					: undefined,
			};
		}),
	);

	function toCssRgb(rgb: [number, number, number]) {
		return `rgb(${rgb.join(', ')})`;
	}

	onMount(() => {
		invoke<OcrThemeOption[]>('get_ocr_themes')
			.then((themes) => (inGameThemes = themes))
			.catch((error) => console.error('Could not load in-game theme colors:', error));
	});
</script>

<CommonSetting
	title="App theme (experimental)"
	description="Choose the colors used by Artus."
	align="vertical"
>
	<Select
		type="single"
		items={selectItems}
		bind:value={config.app_theme}
		onValueChange={() => void updateSetting('app_theme')}
		placeholder="Select an app theme"
	/>
</CommonSetting>
