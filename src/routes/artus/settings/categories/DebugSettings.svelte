<script lang="ts">
	import DictionarySettings from '../components/DictionarySettings.svelte';
	import OcrBoundingBoxesSetting from '../components/OcrBoundingBoxesSetting.svelte';
	import WordGroupingSettings from '../components/WordGroupingSettings.svelte';
</script>

<div class="flex flex-col gap-8">
	<OcrBoundingBoxesSetting />
	<DictionarySettings />
	<WordGroupingSettings />
</div>
