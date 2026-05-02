<script lang="ts">
	import { Label } from 'bits-ui';
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
</script>

<div class="flex justify-between items-center gap-1">
	<div class="flex-1">
		<Label.Root for="ocr-bounding-boxes-toggle">
			<p>Show OCR bounding boxes</p>
			<p class="text-muted-foreground text-xs">
				If enabled, draws red bounding boxes around detected text on the overlay for debugging.
			</p>
		</Label.Root>
	</div>
	<Switch
		id="ocr-bounding-boxes-toggle"
		onCheckedChange={() => updateSetting('show_ocr_bounding_boxes')}
		bind:checked={config.show_ocr_bounding_boxes}
	/>
</div>
