<script lang="ts">
	import DictionaryMappingSetting from '../components/DictionaryMappingSetting.svelte';
	import OverlayDurationSetting from '../components/OverlayDurationSetting.svelte';
	import OverlayModeSetting from '../components/OverlayModeSetting.svelte';
	import WarframeTheme from '../components/WarframeTheme.svelte';

	let overlayToggleMode = $state(false);
</script>

<WarframeTheme />
<OverlayModeSetting bind:mode={overlayToggleMode} />
<OverlayDurationSetting {overlayToggleMode} />
<DictionaryMappingSetting />
