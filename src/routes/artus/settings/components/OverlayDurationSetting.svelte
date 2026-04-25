<script lang="ts">
	import { onMount } from 'svelte';

	import {
		MAX_OVERLAY_DURATION_SECS,
		MIN_OVERLAY_DURATION_SECS,
		getOverlayDurationSecs,
		setOverlayDurationSecs,
	} from '../settings-api';

	let { overlayToggleMode = false }: { overlayToggleMode?: boolean } = $props();

	let isLoading = $state(true);
	let overlayDurationInput = $state(10);
	let status = $state<string | null>(null);

	async function loadDuration() {
		isLoading = true;
		status = null;

		try {
			overlayDurationInput = await getOverlayDurationSecs();
		} catch (error) {
			status = String(error);
		} finally {
			isLoading = false;
		}
	}

	async function saveDuration() {
		status = null;

		const parsed = Number(overlayDurationInput);
		const normalized = Number.isFinite(parsed) ? Math.trunc(parsed) : Number.NaN;

		if (
			!Number.isFinite(normalized) ||
			normalized < MIN_OVERLAY_DURATION_SECS ||
			normalized > MAX_OVERLAY_DURATION_SECS
		) {
			status = `Duration must be between ${MIN_OVERLAY_DURATION_SECS} and ${MAX_OVERLAY_DURATION_SECS} seconds.`;
			return;
		}

		try {
			const savedSeconds = await setOverlayDurationSecs(normalized);
			overlayDurationInput = savedSeconds;
			status = `Overlay duration set to ${savedSeconds}s`;
		} catch (error) {
			status = String(error);
		}
	}

	onMount(() => {
		void loadDuration();
	});
</script>

<div class="mt-6">
	<p class="text-sm">Overlay Duration (seconds)</p>
	<div class="flex items-center gap-2 mt-2">
		<input
			type="number"
			min={MIN_OVERLAY_DURATION_SECS}
			max={MAX_OVERLAY_DURATION_SECS}
			step="1"
			class="px-2 py-1 border rounded w-full"
			disabled={isLoading || overlayToggleMode}
			bind:value={overlayDurationInput}
		/>
		<button
			class="px-3 py-1 border rounded"
			disabled={isLoading || overlayToggleMode}
			onclick={saveDuration}
		>
			Save
		</button>
	</div>
	<p class="mt-1 text-muted-foreground text-xs">
		Range: {MIN_OVERLAY_DURATION_SECS}-{MAX_OVERLAY_DURATION_SECS} seconds.
		{overlayToggleMode ? ' Disabled while Toggle mode is enabled.' : ''}
	</p>
</div>

{#if status}
	<p class="mt-2 text-muted-foreground text-xs">{status}</p>
{/if}
