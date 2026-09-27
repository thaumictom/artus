<script lang="ts">
	import Icon from '@iconify/svelte';
	let { name, mastered = false, ownedCount = 0 }: { name: string; mastered?: boolean; ownedCount?: number } = $props();
</script>

<span class="inline-flex flex-wrap items-center gap-x-2 gap-y-1">
	<span>{name}</span>
	{#if ownedCount > 0}
		<span class="inline-flex items-center gap-2 text-muted-foreground text-xs">
			<span aria-hidden="true" class="h-3 w-px bg-border-secondary"></span>
			{ownedCount} owned
		</span>
	{/if}
	{#if mastered}
		<span class="inline-flex items-center gap-2 text-accent text-xs font-medium">
			<span aria-hidden="true" class="h-3 w-px bg-border-secondary"></span>
			<span class="inline-flex items-center gap-1">
				<Icon icon="material-symbols:check-circle-rounded" class="size-3.5" />
				Mastered
			</span>
		</span>
	{/if}
</span>
