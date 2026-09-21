<script lang="ts">
	import { config, updateSetting } from '$lib/settings.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import Switch from '$lib/components/Switch.svelte';

	const detectionSetting = 'relic_reward_detection';
	const soundSetting = 'relic_reward_sound';
</script>

<CommonSetting
	title="Automatic relic reward detection"
	description="Automatically shows the overlay when the relic reward screen appears."
>
	<Switch
		id="relic-reward-detection-toggle"
		onCheckedChange={() => updateSetting(detectionSetting)}
		bind:checked={config[detectionSetting]}
	/>
</CommonSetting>

<CommonSetting
	title="Relic detection sound"
	description="Play a sound when automatic relic reward detection is triggered."
	disabled={!config.relic_reward_detection}
>
	<Switch
		id="relic-reward-sound-toggle"
		onCheckedChange={() => updateSetting(soundSetting)}
		bind:checked={config[soundSetting]}
	/>
</CommonSetting>
