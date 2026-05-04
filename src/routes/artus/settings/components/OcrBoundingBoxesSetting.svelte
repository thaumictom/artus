<script lang="ts">
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';

	const mainSetting = 'show_ocr_bounding_boxes';
</script>

<CommonSetting
	title="OCR bounding boxes"
	description="If enabled, draws red bounding boxes around detected text on the overlay for debugging"
	labelProps={{ for: mainSetting }}
>
	<Switch
		id={mainSetting}
		onCheckedChange={() => updateSetting(mainSetting)}
		bind:checked={config[mainSetting]}
	/>
</CommonSetting>
