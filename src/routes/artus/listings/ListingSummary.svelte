<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Listing } from './types';

	let { orders }: { orders: Listing[] } = $props();
	let visibleCount = $derived(orders.filter((order) => order.visible).length);
	let sellCount = $derived(orders.filter((order) => order.type === 'sell').length);

	const cards = $derived([
		{ label: 'Total listings', value: orders.length, icon: 'lucide:tags', color: 'bg-sky-500/15 text-sky-400' },
		{ label: 'Visible', value: visibleCount, icon: 'lucide:circle-check', color: 'bg-accent/15 text-accent' },
		{ label: 'Hidden', value: orders.length - visibleCount, icon: 'lucide:pause', color: 'bg-amber-500/15 text-amber-400' },
		{ label: 'Sell listings', value: sellCount, icon: 'lucide:store', color: 'bg-violet-500/15 text-violet-400' },
	]);
</script>

<section aria-label="Listing summary" class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
	{#each cards as card (card.label)}
		<div class="flex items-center gap-4 border border-border-secondary bg-card/50 p-4">
			<div class={`flex size-12 shrink-0 items-center justify-center rounded-full ${card.color}`}>
				<Icon icon={card.icon} class="size-6" />
			</div>
			<div>
				<div class="text-2xl font-bold tabular-nums">{card.value}</div>
				<div class="text-muted-foreground text-sm">{card.label}</div>
			</div>
		</div>
	{/each}
</section>
