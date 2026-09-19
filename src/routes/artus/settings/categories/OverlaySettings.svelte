<script lang="ts">
	import RelatedPriceSettings from '../components/RelatedPriceSettings.svelte';
	import CaptureModsSetting from '../components/CaptureModsSetting.svelte';
	import ThresholdSettings from '../components/ThresholdSettings.svelte';
</script>

<div class="flex flex-col gap-8">
	<RelatedPriceSettings />
	<ThresholdSettings />
	<CaptureModsSetting />
</div>
