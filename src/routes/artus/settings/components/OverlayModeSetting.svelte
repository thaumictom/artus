<script lang="ts">
	import { onMount } from 'svelte';

	import { getOverlayToggleMode, setOverlayToggleMode } from '../settings-api';

	let { mode = $bindable(false) }: { mode?: boolean } = $props();

	let isLoading = $state(true);
	let status = $state<string | null>(null);

	async function loadMode() {
		isLoading = true;
		status = null;

		try {
			mode = await getOverlayToggleMode();
		} catch (error) {
			status = String(error);
		} finally {
			isLoading = false;
		}
	}

	async function saveMode(enabled: boolean = mode) {
		status = null;

		try {
			const savedMode = await setOverlayToggleMode(enabled);
			mode = savedMode;
			status = savedMode
				? 'Overlay mode set to Toggle (press hotkey again to hide).'
				: 'Overlay mode set to Timer.';
		} catch (error) {
			status = String(error);
		}
	}

	onMount(() => {
		void loadMode();
	});
</script>

<div class="mt-6 p-3 border rounded">
	<p class="font-medium text-sm">Overlay Mode</p>
	<label class="flex items-center gap-2 mt-2 text-sm cursor-pointer">
		<input
			type="checkbox"
			disabled={isLoading}
			bind:checked={mode}
			onchange={() => void saveMode(mode)}
		/>
		<span>Toggle mode (press once to show, press again to hide)</span>
	</label>
	<p class="mt-1 text-muted-foreground text-xs">
		{mode
			? 'Toggle mode ignores the timer and keeps the overlay visible until the next hotkey press.'
			: 'Timer mode auto-hides the overlay after the configured duration.'}
	</p>
</div>

{#if status}
	<p class="mt-2 text-muted-foreground text-xs">{status}</p>
{/if}
