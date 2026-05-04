<script lang="ts">
	import { config, updateSetting } from '$lib/settings.svelte';
	import Slider from '$lib/components/Slider.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
</script>

<CommonSetting
	title="Max X Gap Multiplie"
	description="Maximum horizontal gap between two words to be grouped together"
	align="vertical"
>
	<Slider
		min={0.1}
		max={5.0}
		step={0.1}
		type="single"
		onValueCommit={() => updateSetting('ocr_max_x_gap_multiplier')}
		bind:value={config.ocr_max_x_gap_multiplier}
	>
		{#snippet thumbLabel({ value })}
			{value}x
		{/snippet}
	</Slider>
</CommonSetting>
<CommonSetting
	title="Max Y Gap Multiplier"
	description="Maximum vertical gap between two lines of text to be grouped."
	align="vertical"
>
	<Slider
		min={0.1}
		max={5.0}
		step={0.1}
		type="single"
		onValueCommit={() => updateSetting('ocr_max_y_gap_multiplier')}
		bind:value={config.ocr_max_y_gap_multiplier}
	>
		{#snippet thumbLabel({ value })}
			{value}x
		{/snippet}
	</Slider>
</CommonSetting>
<CommonSetting
	title="Vertical Column Tolerance"
	description="Allowed horizontal center-point drift for words stacked vertically."
	align="vertical"
>
	<Slider
		min={0.1}
		max={5.0}
		step={0.1}
		type="single"
		onValueCommit={() => updateSetting('ocr_vertical_column_tolerance')}
		bind:value={config.ocr_vertical_column_tolerance}
	>
		{#snippet thumbLabel({ value })}
			{value}x
		{/snippet}
	</Slider>
</CommonSetting>
<CommonSetting
	title="Row Bucket Y Tolerance"
	description="Maximum Y-axis jitter allowed to consider words on the 'same line'."
	align="vertical"
>
	<Slider
		min={0.1}
		max={5.0}
		step={0.1}
		type="single"
		onValueCommit={() => updateSetting('ocr_row_bucket_y_tolerance')}
		bind:value={config.ocr_row_bucket_y_tolerance}
	>
		{#snippet thumbLabel({ value })}
			{value}x
		{/snippet}
	</Slider>
</CommonSetting>
