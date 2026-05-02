<script lang="ts">
	import { Label } from 'bits-ui';
	import { config, updateSetting } from '$lib/settings.svelte';
	import Slider from '$lib/components/Slider.svelte';
	import Switch from '$lib/components/Switch.svelte';

	let dictionaryMappingHardDisabled = $state(false);
	let dictionaryMappingMinThreshold = $state(0);
	let dictionaryMappingMaxThreshold = $state(1);
</script>

<div class="flex flex-col gap-8">
	<div class="flex flex-col gap-1">
		<div class="flex justify-between items-center gap-1">
			<Label.Root for="dictionary-mapping-toggle" class="flex-1">
				<p>Map OCR text to dictionary items</p>
				<p class="text-muted-foreground text-xs">
					If enabled, OCR words are matched against known item names and tags
				</p>
			</Label.Root>
			<Switch
				id="dictionary-mapping-toggle"
				disabled={dictionaryMappingHardDisabled}
				onCheckedChange={() => updateSetting('ocr_dictionary_mapping_enabled')}
				bind:checked={config.ocr_dictionary_mapping_enabled}
			/>
		</div>

		{#if dictionaryMappingHardDisabled}
			<p class="text-muted-foreground text-xs">
				Dictionary mapping is hard-disabled in backend code.
			</p>
		{/if}
	</div>

	<div class="data-[disabled=true]:cursor-not-allowed" data-disabled={!config.ocr_dictionary_mapping_enabled}>
		<div
			class="flex flex-col gap-3 data-[disabled=true]:opacity-50 transition-opacity data-[disabled=true]:pointer-events-none"
			data-disabled={!config.ocr_dictionary_mapping_enabled}
		>
			<div>
				<p>Dictionary match threshold</p>
				<p class="text-muted-foreground text-xs">
					Words below this confidence are removed from OCR output.
				</p>
			</div>
			<Slider
				min={dictionaryMappingMinThreshold}
				max={dictionaryMappingMaxThreshold}
				step={0.01}
				type="single"
				disabled={!config.ocr_dictionary_mapping_enabled || dictionaryMappingHardDisabled}
				onValueCommit={() => updateSetting('ocr_dictionary_match_threshold')}
				bind:value={config.ocr_dictionary_match_threshold}
			>
				{#snippet thumbLabel({ value })}
					{(typeof value === 'number' ? value : config.ocr_dictionary_match_threshold).toFixed(2)}
				{/snippet}
			</Slider>
		</div>
	</div>
</div>
