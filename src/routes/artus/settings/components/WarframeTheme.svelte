<script lang="ts">
	import { capitalCase } from 'change-case';
	import { config, updateSetting } from '$lib/settings.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import Select from '$lib/components/Select.svelte';
	import type { OcrThemeOption } from '$lib/types';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';

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

<CommonSetting
	title="In-Game Theme"
	description="Select the theme that reflects the in-game theme"
	align="vertical"
>
	<Select
		type="single"
		items={selectItems}
		bind:value={config.ocr_theme}
		onValueChange={() => updateSetting('ocr_theme')}
		placeholder={availableThemes ? 'Select a theme' : 'Loading themes...'}
		disabled={!availableThemes}
	/>
</CommonSetting>
