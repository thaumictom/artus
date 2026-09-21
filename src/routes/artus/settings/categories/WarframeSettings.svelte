<script lang="ts">
	import { platform } from '@tauri-apps/plugin-os';
	import RelicRewardDetection from '../components/RelicRewardDetection.svelte';
	import WarframeTheme from '../components/WarframeTheme.svelte';

	const isWindows = platform() === 'windows';
</script>

<div class="flex flex-col gap-8">
	<WarframeTheme />
	{#if isWindows}
		<RelicRewardDetection />
	{/if}
</div>
