<script>
	import HotkeyCapture from '../components/HotkeyCapture.svelte';
</script>

<div class="flex flex-col gap-8">
	<div>
		<p class="mb-1">Screenshot</p>
		<HotkeyCapture tauriHotkey="screenshot" />
	</div>

	<div>
		<p class="mb-1">Screenshot + add to inventory</p>
		<HotkeyCapture tauriHotkey="screenshot_add_inventory" />
	</div>
</div>
