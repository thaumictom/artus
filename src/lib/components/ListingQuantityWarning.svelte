<script lang="ts">
	import Icon from '@iconify/svelte';

	let { quantity, ownedCount }: { quantity: number; ownedCount: number } = $props();
</script>

{#if quantity > ownedCount}
	<div class="flex items-center gap-1 text-muted-foreground text-sm">
		<Icon icon="mdi:alert-circle-outline" class="size-4" />
		{#if ownedCount === 0}
			<p>
				You do not own <span class="text-amber-400">any</span>
				of this item.
			</p>
		{:else}
			<p>
				You only own <span class="text-amber-400">{ownedCount}</span>
				of this item.
			</p>
		{/if}
	</div>
{/if}
