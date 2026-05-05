<script lang="ts">
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';

	const setting = 'cleanup_on_focus_loss';
</script>

<CommonSetting
	title="Clean up on focus loss"
	description="When Warframe loses focus, the overlay will be fully cleaned up and canceled instead of just visually hiding."
	align="horizontal"
	labelProps={{ for: setting }}
	disabled={config.use_window_ownership || !config.hide_overlay_on_focus_loss}
>
	<Switch
		id={setting}
		onCheckedChange={() => updateSetting(setting)}
		bind:checked={config[setting]}
		disabled={config.use_window_ownership || !config.hide_overlay_on_focus_loss}
	/>
</CommonSetting>
