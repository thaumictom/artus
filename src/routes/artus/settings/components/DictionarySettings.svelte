<script lang="ts">
	import { onMount } from 'svelte';
	import { Label } from 'bits-ui';

	import Slider from '$lib/components/Slider.svelte';
	import Switch from '$lib/components/Switch.svelte';
	import {
		getOcrDictionaryMappingSettings,
		setOcrDictionaryMappingEnabled,
		setOcrDictionaryMatchThreshold,
	} from '../settings-api';

	let isLoading = $state(true);
	let dictionaryMappingEnabled = $state(true);
	let dictionaryMappingThreshold = $state(0.62);
	let dictionaryMappingHardDisabled = $state(false);
	let dictionaryMappingMinThreshold = $state(0);
	let dictionaryMappingMaxThreshold = $state(1);
	let status = $state<string | null>(null);

	async function loadSettings() {
		isLoading = true;
		status = null;

		try {
			const settings = await getOcrDictionaryMappingSettings();
			dictionaryMappingEnabled = settings.enabled;
			dictionaryMappingThreshold = settings.threshold;
			dictionaryMappingHardDisabled = settings.hard_disabled;
			dictionaryMappingMinThreshold = settings.min_threshold;
			dictionaryMappingMaxThreshold = settings.max_threshold;
		} catch (error) {
			status = String(error);
		} finally {
			isLoading = false;
		}
	}

	async function saveEnabled(enabled: boolean = dictionaryMappingEnabled) {
		status = null;

		try {
			const savedEnabled = await setOcrDictionaryMappingEnabled(enabled);
			dictionaryMappingEnabled = savedEnabled;
			status = savedEnabled ? 'Dictionary mapping enabled.' : 'Dictionary mapping disabled.';
		} catch (error) {
			status = String(error);
		}
	}

	async function saveThreshold(threshold: number = dictionaryMappingThreshold) {
		status = null;

		const parsed = Number(threshold);
		if (
			!Number.isFinite(parsed) ||
			parsed < dictionaryMappingMinThreshold ||
			parsed > dictionaryMappingMaxThreshold
		) {
			status = `Threshold must be between ${dictionaryMappingMinThreshold} and ${dictionaryMappingMaxThreshold}.`;
			return;
		}

		try {
			const savedThreshold = await setOcrDictionaryMatchThreshold(parsed);
			dictionaryMappingThreshold = savedThreshold;
			status = `Dictionary threshold set to ${savedThreshold.toFixed(2)}.`;
		} catch (error) {
			status = String(error);
		}
	}

	onMount(() => {
		void loadSettings();
	});
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
				disabled={isLoading || dictionaryMappingHardDisabled}
				onCheckedChange={(nextEnabled) => void saveEnabled(nextEnabled)}
				checked={dictionaryMappingEnabled}
			/>
		</div>

		{#if dictionaryMappingHardDisabled}
			<p class="text-muted-foreground text-xs">
				Dictionary mapping is hard-disabled in backend code.
			</p>
		{/if}
	</div>

	<div class="flex flex-col gap-6">
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
			disabled={!dictionaryMappingEnabled || dictionaryMappingHardDisabled || isLoading}
			onValueCommit={() => void saveThreshold(dictionaryMappingThreshold)}
			bind:value={dictionaryMappingThreshold}
		>
			{#snippet thumbLabel({ value })}
				{(typeof value === 'number' ? value : dictionaryMappingThreshold).toFixed(2)}
			{/snippet}
		</Slider>
	</div>
</div>
