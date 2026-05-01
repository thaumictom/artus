<script lang="ts">
	import type { ItemSchema } from '$lib/schemas';
	import type z from 'zod';

	let { itemData }: { itemData: z.infer<typeof ItemSchema> } = $props();
</script>

<div class="flex items-center gap-4 p-4 border w-full max-w-2xl">
	<img
		src={`https://warframe.market/static/assets/${itemData.i18n?.en.icon}`}
		alt={itemData.i18n?.en.name}
		class="bg-surface h-20 object-contain aspect-square text-transparent"
	/>
	<div class="flex flex-col">
		<h1 class="font-medium text-lg">{itemData.i18n?.en.name}</h1>
		{#if itemData.vaulted}
			<div class="text-muted-foreground text-xs uppercase">Vaulted</div>
		{/if}
		{#if itemData.ducats}
			<div class="text-muted-foreground text-xs uppercase">{itemData.ducats} Ducats</div>
		{/if}
	</div>
</div>
