<script lang="ts">
	import { onMount } from 'svelte';
	import { Label } from 'bits-ui';
	import Switch from '$lib/components/Switch.svelte';
	import { getShowOcrBoundingBoxes, setShowOcrBoundingBoxes } from '../settings-api';

	let isLoading = $state(true);
	let showOcrBoundingBoxes = $state(false);

	async function load() {
		isLoading = true;
		try {
			showOcrBoundingBoxes = await getShowOcrBoundingBoxes();
		} catch (error) {
			console.error('Failed to load OCR bounding boxes setting:', error);
		} finally {
			isLoading = false;
		}
	}

	async function save(enabled: boolean) {
		try {
			showOcrBoundingBoxes = await setShowOcrBoundingBoxes(enabled);
		} catch (error) {
			console.error('Failed to save OCR bounding boxes setting:', error);
		}
	}

	onMount(() => {
		void load();
	});
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
		disabled={isLoading}
		onCheckedChange={(enabled) => void save(enabled)}
		checked={showOcrBoundingBoxes}
	/>
</div>
