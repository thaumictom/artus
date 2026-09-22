<script lang="ts">
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import Switch from '$lib/components/Switch.svelte';
	import { requestDesktopNotificationPermission } from '$lib/notifications.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';

	let permissionError = $state(false);

	async function setEnabled(enabled: boolean) {
		config.desktop_notifications_enabled = enabled;
		permissionError = false;
		await updateSetting('desktop_notifications_enabled');
		if (enabled) permissionError = !(await requestDesktopNotificationPermission());
	}
</script>

<CommonSetting
	title="Desktop notifications"
	description="Show native Windows or Linux notifications for matching world-state rules. Notifications always remain available in the Artus notification center."
>
	<Switch
		id="desktop-notifications-toggle"
		checked={config.desktop_notifications_enabled}
		onCheckedChange={(enabled) => void setEnabled(enabled)}
	/>
</CommonSetting>
{#if permissionError}
	<p role="alert" class="mt-2 text-danger text-xs">
		Notification permission was not granted by the operating system.
	</p>
{/if}
