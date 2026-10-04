<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Listing } from './types';

	let { orders }: { orders: Listing[] } = $props();
	let visibleCount = $derived(orders.filter((order) => order.visible).length);
</script>

<section class="flex items-center divide-border-secondary divide-x">
	<div class="pr-4">
		{orders.length} listings
	</div>
	<div class="px-4 text-muted-foreground text-base">
		{visibleCount} visible
	</div>
	<div class="pl-4 text-muted-foreground text-base">
		{orders.length - visibleCount} hidden
	</div>
</section>
