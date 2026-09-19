<script lang="ts">
	import CleanupOnFocusLossSetting from '../components/CleanupOnFocusLossSetting.svelte';
	import HideOverlayOnFocusLossSetting from '../components/HideOverlayOnFocusLossSetting.svelte';
	import OverlayDurationSetting from '../components/OverlayDurationSetting.svelte';
	import UseWindowOwnershipSetting from '../components/UseWindowOwnershipSetting.svelte';
</script>

<div class="flex flex-col gap-8">
	<HideOverlayOnFocusLossSetting />
	<CleanupOnFocusLossSetting />
	<UseWindowOwnershipSetting />
	<OverlayDurationSetting />
</div>
