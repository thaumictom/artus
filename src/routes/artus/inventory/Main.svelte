<script lang="ts">
	import { columns, type WarframeItem } from './columns';
	import { LazyStore } from '@tauri-apps/plugin-store';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';

	const store = new LazyStore('inventory.json');

	let data = $state<WarframeItem[]>([]);
	
	onMount(() => {
		let unlisten: UnlistenFn | undefined;

		(async () => {
			const saved = await store.get<WarframeItem[]>('items');
			if (saved) {
				data = saved;
			}

			unlisten = await listen<{ words: any[]; is_inventory_add?: boolean }>(
				'ocr_result',
				(event) => {
					if (!event.payload.is_inventory_add) return;

					const words = event.payload.words;
					let addedAny = false;

					for (const word of words) {
						if (!word.slug) continue;
						
						const itemName = word.text;
						const median = word.market_median;
						const ducats = word.ducats ?? 0;

						const existing = data.find((i) => i.name === itemName);
						if (existing) {
							existing.quantity += 1;
						} else {
							data.push({
								name: itemName,
								quantity: 1,
								marketMedian: median,
								ducats: ducats,
							});
						}
						addedAny = true;
					}

					if (addedAny) {
						saveInventory();
					}
				}
			);
		})();

		return () => {
			if (unlisten) unlisten();
		};
	});

	async function saveInventory() {
		await store.set('items', $state.snapshot(data));
		await store.save();
	}

	function updateQuantity(item: WarframeItem, delta: number) {
		item.quantity += delta;
		if (item.quantity <= 0) {
			data = data.filter((i) => i !== item);
		}
		saveInventory();
	}

	const totalPlatinum = $derived(data.reduce((sum, item) => sum + (item.marketMedian ?? 0) * item.quantity, 0));
	const totalDucats = $derived(data.reduce((sum, item) => sum + item.ducats * item.quantity, 0));
</script>

<div class="h-full w-full flex flex-col p-6 bg-background text-foreground overflow-y-auto">
	<div class="mb-6 flex justify-between items-end">
		<h1 class="text-3xl font-bold font-stretch-condensed text-foreground/90 uppercase tracking-tighter">Inventory</h1>
	</div>

	<div class="border rounded-md overflow-hidden bg-card/50 shadow-sm flex-1 mb-6">
		<table class="w-full text-sm text-left">
			<thead class="bg-muted text-muted-foreground uppercase text-xs">
				<tr>
					{#each columns as col}
						<th class="px-4 py-3 font-medium tracking-wide">
							{col.header}
						</th>
					{/each}
				</tr>
			</thead>
			<tbody class="divide-y border-t">
				{#each data as item}
					<tr class="hover:bg-muted/30 transition-colors">
						{#each columns as col}
							<td class="px-4 py-2.5">
								{#if col.id === 'actions'}
									<div class="flex gap-1">
										<button class="bg-secondary text-secondary-foreground hover:bg-secondary/80 rounded px-2 py-1.5 transition-colors cursor-pointer" onclick={() => updateQuantity(item, -1)}>
											<Icon icon="lucide:minus" class="size-3" />
										</button>
										<button class="bg-primary text-primary-foreground hover:bg-primary/90 rounded px-2 py-1.5 transition-colors cursor-pointer" onclick={() => updateQuantity(item, 1)}>
											<Icon icon="lucide:plus" class="size-3" />
										</button>
									</div>
								{:else if col.accessorKey === 'marketMedian'}
									<div class="flex items-center gap-1.5 text-amber-500/90 font-medium">
										{item.marketMedian !== undefined ? Math.round(item.marketMedian) : '-'}
										<img src="/icons/platinum.png" class="size-3.5" alt="pt" />
									</div>
								{:else if col.id === 'totalPrice'}
									<div class="flex items-center gap-1.5 text-amber-500/90 font-bold">
										{Math.round((item.marketMedian ?? 0) * item.quantity)}
										<img src="/icons/platinum.png" class="size-3.5" alt="pt" />
									</div>
								{:else if col.accessorKey === 'ducats'}
									<div class="flex items-center gap-1.5 text-cyan-500/90 font-medium">
										{item.ducats || '-'}
										<img src="/icons/ducats.png" class="size-3.5" alt="ducats" />
									</div>
								{:else if col.id === 'totalDucats'}
									<div class="flex items-center gap-1.5 text-cyan-500/90 font-bold">
										{item.ducats * item.quantity}
										<img src="/icons/ducats.png" class="size-3.5" alt="ducats" />
									</div>
								{:else if col.accessorKey === 'name'}
									<div class="font-medium text-foreground/90">{item.name}</div>
								{:else if col.accessorKey === 'quantity'}
									<div class="font-mono text-xs bg-muted/50 px-2 py-0.5 rounded-full inline-block">x{item.quantity}</div>
								{:else}
									{item[col.accessorKey as keyof typeof item]}
								{/if}
							</td>
						{/each}
					</tr>
				{:else}
					<tr>
						<td colspan={columns.length} class="px-4 py-8 text-center text-muted-foreground">
							Your inventory is empty. Press <span class="font-mono bg-muted px-1 py-0.5 rounded text-xs">Ctrl+Shift+Home</span> in Warframe to scan items.
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>

	<!-- Footer totals -->
	<div class="p-5 border rounded-md bg-card shadow-sm flex justify-between items-center bg-gradient-to-r from-card to-muted/20 border-l-4 border-l-primary/50">
		<span class="text-xl font-bold font-stretch-condensed text-foreground/80 uppercase tracking-wide">Grand Total</span>
		<div class="flex gap-8">
			<div class="flex items-center gap-2.5">
				<span class="text-2xl font-bold text-amber-500/90 tracking-tighter">{Math.round(totalPlatinum)}</span>
				<img src="/icons/platinum.png" class="size-6" alt="Platinum" />
			</div>
			<div class="flex items-center gap-2.5">
				<span class="text-2xl font-bold text-cyan-500/90 tracking-tighter">{totalDucats}</span>
				<img src="/icons/ducats.png" class="size-6" alt="Ducats" />
			</div>
		</div>
	</div>
</div>