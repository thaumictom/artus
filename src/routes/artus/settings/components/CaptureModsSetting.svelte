<script lang="ts">
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';

	const setting = 'capture_mods';
</script>

<CommonSetting
	title="Capture mods"
	description="Add more color targets to also capture mods. Result can become more inaccurate when enabled. Relic reward screen is unaffected by this setting."
	labelProps={{ for: setting }}
>
	<Switch
		id={setting}
		onCheckedChange={() => updateSetting(setting)}
		bind:checked={config[setting]}
	/>
</CommonSetting>
