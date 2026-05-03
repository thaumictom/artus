<script lang="ts">
	import OverlayDurationSetting from '../components/OverlayDurationSetting.svelte';
	import CaptureModsSetting from '../components/CaptureModsSetting.svelte';
	import HideOverlayOnFocusLossSetting from '../components/HideOverlayOnFocusLossSetting.svelte';
</script>

<div class="flex flex-col gap-8">
	<HideOverlayOnFocusLossSetting />
	<OverlayDurationSetting />
	<CaptureModsSetting />
</div>
