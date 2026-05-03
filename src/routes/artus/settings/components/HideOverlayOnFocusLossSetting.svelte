<script lang="ts">
	import { Label } from 'bits-ui';
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
</script>

<div class="flex justify-between items-center gap-4">
	<Label.Root for="hide-overlay-focus-toggle" class="flex-1">
		<p>Hide Overlay on Focus Loss</p>
		<p class="text-muted-foreground text-xs">
			Automatically hide the overlay and cancel OCR processing when Warframe loses focus. Does not affect automatic Relic Reward scans.
		</p>
	</Label.Root>
	<Switch
		id="hide-overlay-focus-toggle"
		onCheckedChange={() => updateSetting('hide_overlay_on_focus_loss')}
		bind:checked={config.hide_overlay_on_focus_loss}
	/>
</div>
