<script lang="ts">
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
</script>

<CommonSetting
	title="Hide to tray when closing the app"
	description="Keep Artus running in the tray when you close its window. Use the tray menu to restart or quit."
>
	<Switch
		id="hide-to-tray-on-close"
		checked={config.hide_to_tray_on_close}
		onCheckedChange={(enabled) => {
			config.hide_to_tray_on_close = enabled;
			void updateSetting('hide_to_tray_on_close');
		}}
	/>
</CommonSetting>
