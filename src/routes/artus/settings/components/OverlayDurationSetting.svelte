<script lang="ts">
	import { Label } from 'bits-ui';
	import { config, updateSetting } from '$lib/settings.svelte';

	import Slider from '$lib/components/Slider.svelte';
	import Switch from '$lib/components/Switch.svelte';

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
</script>

<div class="flex flex-col gap-8">
	<div class="flex justify-between items-center gap-1">
		<Label.Root for="overlay-mode-toggle" class="flex-1">
			<p>Toggle overlay with screenshot keybind</p>
			<p class="text-muted-foreground text-xs">
				If enabled, the overlay will be shown until the keybind is pressed again
			</p>
		</Label.Root>
		<Switch
			id="overlay-mode-toggle"
			onCheckedChange={() => updateSetting('overlay_toggle_mode')}
			bind:checked={config.overlay_toggle_mode}
		/>
	</div>

	<div class="data-[disabled=true]:cursor-not-allowed" data-disabled={config.overlay_toggle_mode}>
		<div
			class="flex flex-col gap-3 data-[disabled=true]:opacity-50 transition-opacity data-[disabled=true]:pointer-events-none"
			data-disabled={config.overlay_toggle_mode}
		>
			<div>
				<p>Overlay duration</p>
				<p class="text-muted-foreground text-xs">
					Time in seconds that the overlay will be shown after pressing the keybind
				</p>
			</div>
			<Slider
				min={5}
				max={120}
				step={5}
				type="single"
				disabled={config.overlay_toggle_mode}
				onValueCommit={saveDuration}
				bind:value={config.overlay_duration_secs}
			>
				{#snippet thumbLabel({ value })}
					{value}s
				{/snippet}
			</Slider>
		</div>
	</div>
</div>
