<script lang="ts">
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';

	const setting = 'notification_sound';
</script>

<CommonSetting
	title="Notification sound"
	description="Play the Artus beep when a world-state notification is added."
>
	<Switch
		id="notification-sound-toggle"
		bind:checked={config[setting]}
		onCheckedChange={() => updateSetting(setting)}
	/>
</CommonSetting>
