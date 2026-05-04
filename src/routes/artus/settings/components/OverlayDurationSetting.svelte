<script lang="ts">
	import { Label } from 'bits-ui';
	import { config, updateSetting } from '$lib/settings.svelte';

	import Slider from '$lib/components/Slider.svelte';
	import Switch from '$lib/components/Switch.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';

	let status = $state<string | null>(null);

	function saveDuration() {
		status = null;
		const parsed = Number(config.overlay_duration_secs);
		const normalized = Number.isFinite(parsed) ? Math.trunc(parsed) : Number.NaN;

		if (!Number.isFinite(normalized) || normalized <= 0) {
			status = 'Duration must be a positive number of seconds.';
			return;
		}

		config.overlay_duration_secs = normalized;
		updateSetting('overlay_duration_secs');
	}

	const mainSetting = 'overlay_toggle_mode';
	const durationSetting = 'overlay_duration_secs';
</script>

<CommonSetting
	title="Toggle overlay with screenshot keybind"
	description="If enabled, the overlay will be shown until the keybind is pressed again"
	labelProps={{ for: mainSetting }}
>
	<Switch
		id={mainSetting}
		onCheckedChange={() => updateSetting(mainSetting)}
		bind:checked={config[mainSetting]}
	/>
</CommonSetting>
<CommonSetting
	title="Overlay duration"
	description="Time in seconds that the overlay will be shown after pressing the keybind"
	disabled={config[mainSetting]}
	align="vertical"
>
	<Slider
		min={5}
		max={60}
		step={5}
		type="single"
		onValueCommit={saveDuration}
		bind:value={config[durationSetting]}
	>
		{#snippet thumbLabel({ value })}
			{value}s
		{/snippet}
	</Slider>
</CommonSetting>
