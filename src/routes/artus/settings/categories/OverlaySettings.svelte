<script lang="ts">
	import OverlayDurationSetting from '../components/OverlayDurationSetting.svelte';
	import CaptureModsSetting from '../components/CaptureModsSetting.svelte';
	import HideOverlayOnFocusLossSetting from '../components/HideOverlayOnFocusLossSetting.svelte';
	import CleanupOnFocusLossSetting from '../components/CleanupOnFocusLossSetting.svelte';
	import ThresholdSettings from '../components/ThresholdSettings.svelte';
	import UseWindowOwnershipSetting from '../components/UseWindowOwnershipSetting.svelte';
</script>

<div class="flex flex-col gap-8">
	<HideOverlayOnFocusLossSetting />
	<CleanupOnFocusLossSetting />
	<UseWindowOwnershipSetting />
	<OverlayDurationSetting />
	<ThresholdSettings />
	<CaptureModsSetting />
</div>
