<script lang="ts">
	import { capitalCase } from 'change-case';
	import { config, updateSetting } from '$lib/settings.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import Select from '$lib/components/Select.svelte';
	import type { OcrThemeOption } from '$lib/types';

	let availableThemes = $state<OcrThemeOption[]>();

	let selectItems = $derived(
		availableThemes?.map(({ name }) => ({
			label: capitalCase(name),
			value: name,
		})) ?? [],
	);

	onMount(() => {
		invoke<OcrThemeOption[]>('get_ocr_themes')
			.then((res) => (availableThemes = res))
			.catch((err) => {
				console.error(err);
			});
	});
</script>

<div class="flex flex-col gap-1">
	<div class="mb-1">
		<h2>In-Game Theme</h2>
		<p class="text-muted-foreground text-xs">Select the theme that reflects the in-game theme</p>
	</div>

	<Select
		type="single"
		items={selectItems}
		bind:value={config.ocr_theme}
		onValueChange={() => updateSetting('ocr_theme')}
		placeholder={availableThemes ? 'Select a theme' : 'Loading themes...'}
		disabled={!availableThemes}
	/>
</div>
