<script lang="ts">
	import { platform } from '@tauri-apps/plugin-os';
	import CleanupOnFocusLossSetting from '../components/CleanupOnFocusLossSetting.svelte';
	import HideOverlayOnFocusLossSetting from '../components/HideOverlayOnFocusLossSetting.svelte';
	import OverlayDurationSetting from '../components/OverlayDurationSetting.svelte';
	import UseWindowOwnershipSetting from '../components/UseWindowOwnershipSetting.svelte';

	const isWindows = platform() === 'windows';
</script>

<div class="flex flex-col gap-8">
	<HideOverlayOnFocusLossSetting />
	<CleanupOnFocusLossSetting />
	{#if isWindows}
		<UseWindowOwnershipSetting />
	{/if}
	<OverlayDurationSetting />
</div>
