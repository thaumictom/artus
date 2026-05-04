<script lang="ts">
	import { Label } from 'bits-ui';
	import { config, updateSetting } from '$lib/settings.svelte';
	import Slider from '$lib/components/Slider.svelte';
	import Switch from '$lib/components/Switch.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';

	const mainSetting = 'ocr_dictionary_mapping_enabled';
	const thresholdSetting = 'ocr_dictionary_match_threshold';
</script>

<CommonSetting
	title="Dictionary mapping"
	description="If enabled, OCR words are matched against known item names and tags"
	labelProps={{ for: mainSetting }}
>
	<Switch
		id={mainSetting}
		onCheckedChange={() => updateSetting(mainSetting)}
		bind:checked={config[mainSetting]}
	/>
</CommonSetting>
<CommonSetting
	title="Dictionary match threshold"
	description="Words below this confidence are removed from OCR output"
	disabled={!config[mainSetting]}
	align="vertical"
>
	<Slider
		min={0}
		max={1}
		step={0.01}
		type="single"
		onValueCommit={() => updateSetting(thresholdSetting)}
		bind:value={config[thresholdSetting]}
	>
		{#snippet thumbLabel({ value })}
			{(typeof value === 'number' ? value : config[thresholdSetting]).toFixed(2)}
		{/snippet}
	</Slider>
</CommonSetting>
