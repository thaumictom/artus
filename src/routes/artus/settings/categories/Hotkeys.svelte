<script lang="ts">
	import HotkeyCapture from '../components/HotkeyCapture.svelte';
	import { config } from '$lib/settings.svelte';
	const overlayHotkeys: [keyof typeof config.hotkeys, string][] = [
		['cycle', 'Cycle items'],
		['cycle_back', 'Cycle items backward'],
		['navigate_up', 'Navigate up'],
		['navigate_left', 'Navigate left'],
		['navigate_down', 'Navigate down'],
		['navigate_right', 'Navigate right'],
		['inventory_decrement', 'Remove one from inventory'],
		['inventory_increment', 'Add one to inventory'],
		['inventory_add_all', 'Add all visible items to inventory'],
	];
</script>

<div class="flex flex-col gap-8">
	<div>
		<p class="mb-1">Screenshot</p>
		<HotkeyCapture tauriHotkey="screenshot" />
	</div>

	<div>
		<p class="mb-1">Screenshot + mark mastery items</p>
		<HotkeyCapture tauriHotkey="screenshot_add_mastery" />
	</div>
	{#each overlayHotkeys as [action, label]}
		<div>
			<p class="mb-1">{label}</p>
			<HotkeyCapture tauriHotkey={action} />
		</div>
	{/each}
</div>
