<script lang="ts">
	import { platform } from '@tauri-apps/plugin-os';
	import { config, updateRelicDetectionSetting, updateSetting } from '$lib/settings.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import Switch from '$lib/components/Switch.svelte';

	const detectionSetting = 'relic_reward_detection';
	const visualDetectionSetting = 'visual_relic_reward_detection';
	const soundSetting = 'relic_reward_sound';
	const isWindows = platform() === 'windows';
	const automaticDetectionEnabled = $derived(
		config.relic_reward_detection || config.visual_relic_reward_detection,
	);

	function updateDetection(
		setting: typeof detectionSetting | typeof visualDetectionSetting,
		enabled: boolean,
	) {
		void updateRelicDetectionSetting(setting, enabled);
	}
</script>

{#if isWindows}
	<CommonSetting
		title="Automatic relic reward detection"
		description="Use Windows debugging tool to detect the relic reward screen."
	>
		<Switch
			id="relic-reward-detection-toggle"
			onCheckedChange={(enabled) => updateDetection(detectionSetting, enabled)}
			bind:checked={config[detectionSetting]}
		/>
	</CommonSetting>
{/if}

{#if isWindows}
	<CommonSetting
		title="Add selected relic reward to inventory"
		description="Automatically add the selected reward when the relic screen closes."
		disabled={!config.relic_reward_detection}
	>
		<Switch
			id="relic-reward-auto-add-toggle"
			onCheckedChange={() => updateSetting('relic_reward_auto_add')}
			bind:checked={config.relic_reward_auto_add}
		/>
	</CommonSetting>
{/if}

<CommonSetting
	title="Visual relic detection (experimental)"
	description="Continuously checks the Warframe reward area for a stable row of recognized relic rewards."
>
	<Switch
		id="visual-relic-reward-detection-toggle"
		onCheckedChange={(enabled) => updateDetection(visualDetectionSetting, enabled)}
		bind:checked={config[visualDetectionSetting]}
	/>
</CommonSetting>

<CommonSetting
	title="Relic detection sound"
	description="Play a sound when automatic relic reward detection is triggered."
	disabled={!automaticDetectionEnabled}
>
	<Switch
		id="relic-reward-sound-toggle"
		onCheckedChange={() => updateSetting(soundSetting)}
		bind:checked={config[soundSetting]}
	/>
</CommonSetting>
