<script lang="ts">
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';

	const setting = 'use_window_ownership';
</script>

<CommonSetting
	title="Use window ownership (Sandwich Effect)"
	description="Attach the overlay to Warframe. This prevents it from floating over other apps when alt-tabbed, but is risky due to Anti-Cheat detection. Keep disabled to stay safe."
	align="horizontal"
	labelProps={{ for: setting }}
>
	<Switch
		id={setting}
		onCheckedChange={() => updateSetting(setting)}
		bind:checked={config[setting]}
	/>
</CommonSetting>
