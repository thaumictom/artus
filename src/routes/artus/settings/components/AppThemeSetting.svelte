<script lang="ts">
	import { appThemes } from '$lib/app-themes';
	import { config, updateSetting } from '$lib/settings.svelte';
	import Select from '$lib/components/Select.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';

	const selectItems = appThemes.map(({ value, label, swatchColors }) => ({
		value,
		label,
		swatchColors: [...swatchColors] as [string, string],
	}));
</script>

<CommonSetting title="App theme" description="Choose the colors used by Artus." align="vertical">
	<Select
		type="single"
		items={selectItems}
		bind:value={config.app_theme}
		onValueChange={() => void updateSetting('app_theme')}
		placeholder="Select an app theme"
	/>
</CommonSetting>
