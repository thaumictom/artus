<script lang="ts">
	import { worldStateLabel, type WorldStateMapping } from '$lib/worldstate-labels';
	let { category, code, fallback = 'Unknown' }: {
		category: WorldStateMapping;
		code: string;
		fallback?: string;
	} = $props();
</script>

{worldStateLabel(category, code, fallback)}
