<script lang="ts">
	import { onMount } from 'svelte';

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

<div class="mt-6 p-3 border rounded">
	<p class="font-medium text-sm">Dictionary Mapping</p>
	<label class="flex items-center gap-2 mt-2 text-sm cursor-pointer">
		<input
			type="checkbox"
			disabled={isLoading || dictionaryMappingHardDisabled}
			bind:checked={dictionaryMappingEnabled}
			onchange={() => void saveEnabled(dictionaryMappingEnabled)}
		/>
		<span>Map OCR text to dictionary items</span>
	</label>

	{#if dictionaryMappingHardDisabled}
		<p class="mt-1 text-muted-foreground text-xs">
			Dictionary mapping is hard-disabled in backend code.
		</p>
	{/if}

	<div class="mt-3">
		<p class="text-muted-foreground text-xs">Threshold: {dictionaryMappingThreshold.toFixed(2)}</p>
		<div class="flex items-center gap-2 mt-2">
			<input
				type="range"
				class="w-full"
				min={dictionaryMappingMinThreshold}
				max={dictionaryMappingMaxThreshold}
				step="0.01"
				disabled={!dictionaryMappingEnabled || dictionaryMappingHardDisabled || isLoading}
				bind:value={dictionaryMappingThreshold}
			/>
			<button
				class="px-3 py-1 border rounded"
				disabled={!dictionaryMappingEnabled || dictionaryMappingHardDisabled || isLoading}
				onclick={() => void saveThreshold(dictionaryMappingThreshold)}
			>
				Save
			</button>
		</div>
		<p class="mt-1 text-muted-foreground text-xs">
			Words below this confidence are removed from OCR output.
		</p>
	</div>
</div>

{#if status}
	<p class="mt-2 text-muted-foreground text-xs">{status}</p>
{/if}
