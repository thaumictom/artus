<script lang="ts">
	import { Label } from 'bits-ui';
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
</script>

<div class="flex justify-between items-center gap-1">
	<div class="flex-1">
		<Label.Root for="capture-mods-toggle">
			<p>Capture Mods</p>
			<p class="text-muted-foreground text-xs">
				If enabled, the manual OCR capture will also capture and identify mods (Gold, Silver, Bronze, Archon, Special). This does not affect automatic relic reward capture.
			</p>
		</Label.Root>
	</div>
	<Switch
		id="capture-mods-toggle"
		onCheckedChange={() => updateSetting('capture_mods')}
		bind:checked={config.capture_mods}
	/>
</div>
