<script lang="ts">
	import { Label } from 'bits-ui';
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
</script>

<div class="flex justify-between items-center gap-4">
	<Label.Root for="capture-mods-toggle" class="flex-1">
		<p>Capture Mods</p>
		<p class="text-muted-foreground text-xs">
			Add more color targets to also capture mods. Result can be inaccurate. Relic reward screen is
			unaffected by this setting.
		</p>
	</Label.Root>
	<Switch
		id="capture-mods-toggle"
		onCheckedChange={() => updateSetting('capture_mods')}
		bind:checked={config.capture_mods}
	/>
</div>
