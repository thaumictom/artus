<script lang="ts">
	import { onMount } from 'svelte';
	import { Label } from 'bits-ui';

	import Slider from '$lib/components/Slider.svelte';
	import Switch from '$lib/components/Switch.svelte';
	import {
		getOverlayDurationSecs,
		getOverlayToggleMode,
		setOverlayDurationSecs,
		setOverlayToggleMode,
	} from '../settings-api';

	let isModeLoading = $state(true);
	let isDurationLoading = $state(true);
	let overlayToggleMode = $state(false);
	let overlayDurationInput = $state(10);
	let status = $state<string | null>(null);

	async function loadMode() {
		isModeLoading = true;

		try {
			overlayToggleMode = await getOverlayToggleMode();
		} catch (error) {
			status = String(error);
		} finally {
			isModeLoading = false;
		}
	}

	async function loadDuration() {
		isDurationLoading = true;

		try {
			overlayDurationInput = await getOverlayDurationSecs();
		} catch (error) {
			status = String(error);
		} finally {
			isDurationLoading = false;
		}
	}

	async function saveMode(enabled: boolean = overlayToggleMode) {
		status = null;

		try {
			const savedMode = await setOverlayToggleMode(enabled);
			overlayToggleMode = savedMode;
		} catch (error) {
			status = String(error);
		}
	}

	async function saveDuration() {
		status = null;

		const parsed = Number(overlayDurationInput);
		const normalized = Number.isFinite(parsed) ? Math.trunc(parsed) : Number.NaN;

		if (!Number.isFinite(normalized) || normalized <= 0) {
			status = 'Duration must be a positive number of seconds.';
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
		void loadMode();
		void loadDuration();
	});
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
			disabled={isModeLoading}
			onCheckedChange={(mode) => void saveMode(mode)}
			checked={overlayToggleMode}
		/>
	</div>

	<div class="data-[disabled=true]:cursor-not-allowed" data-disabled={overlayToggleMode}>
		<div
			class="flex flex-col gap-3 data-[disabled=true]:opacity-50 transition-opacity data-[disabled=true]:pointer-events-none"
			data-disabled={overlayToggleMode}
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
				disabled={isDurationLoading || isModeLoading || overlayToggleMode}
				onValueCommit={saveDuration}
				bind:value={overlayDurationInput}
			>
				{#snippet thumbLabel({ value })}
					{value}s
				{/snippet}
			</Slider>
		</div>
	</div>
</div>
