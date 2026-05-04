<script lang="ts">
	import DictionarySettings from '../components/DictionarySettings.svelte';
	import OcrBoundingBoxesSetting from '../components/OcrBoundingBoxesSetting.svelte';
	import OcrGroupingSettings from '../components/OcrGroupingSettings.svelte';
</script>

<div class="flex flex-col gap-8">
	<OcrBoundingBoxesSetting />
	<OcrGroupingSettings />
	<DictionarySettings />
</div>
