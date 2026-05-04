<script lang="ts">
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';

	const setting = 'hide_overlay_on_focus_loss';
</script>

<CommonSetting
	title="Hide overlay on focus loss"
	description="When Warframe loses focus, toggle the overlay off. Useful on Linux, where window handling is limited."
	align="horizontal"
	labelProps={{ for: setting }}
>
	<Switch
		id={setting}
		onCheckedChange={() => updateSetting(setting)}
		bind:checked={config[setting]}
	/>
</CommonSetting>
