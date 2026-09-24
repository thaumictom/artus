<script lang="ts">
	import Icon from '@iconify/svelte';
	let { name, mastered = false, ownedCount = 0 }: { name: string; mastered?: boolean; ownedCount?: number } = $props();
</script>

<span class="inline-flex flex-wrap items-center gap-x-2 gap-y-1">
	<span>{name}</span>
	{#if mastered}<span class="inline-flex items-center gap-1 text-accent text-xs font-medium"><Icon icon="material-symbols:check-circle-rounded" class="size-3.5" />Mastered</span>{/if}
	{#if ownedCount > 0}<span class="text-muted-foreground text-xs">{ownedCount} owned</span>{/if}
</span>
