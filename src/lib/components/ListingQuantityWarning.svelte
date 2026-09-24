<script lang="ts">
	let { quantity, ownedCount }: { quantity: number; ownedCount: number } = $props();
</script>

{#if quantity > ownedCount}
	<p role="alert" class="text-amber-400 text-sm">You own {ownedCount}; this listing quantity is higher.</p>
{/if}
