<script lang="ts">
	import Dialog from '$lib/components/Dialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import Switch from '$lib/components/Switch.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import { quicklistPrice, quicklistStrategies, type QuicklistStrategy } from '$lib/quicklist';
	import { config, updateSetting } from '$lib/settings.svelte';

	let open = $state(false);
	const exampleOffers = [12, 13, 17, 18, 20];
	const exampleMedian = 15;
	let examplePrice = $derived(
		quicklistPrice(config.quicklist_price_strategy, exampleMedian, exampleOffers),
	);
	let preview = $derived(
		[
			...exampleOffers.map((price) => ({ price, yours: false })),
			{ price: examplePrice, yours: true },
		].sort((a, b) => a.price - b.price || Number(a.yours) - Number(b.yours)),
	);

	function selectStrategy(strategy: QuicklistStrategy) {
		config.quicklist_price_strategy = strategy;
		void updateSetting('quicklist_price_strategy');
	}
</script>

<div class="flex flex-col gap-8">
	<CommonSetting
		title="Always hide quicklists first"
		description="Create quicklist orders as hidden listings until you make them visible."
		labelProps={{ for: 'quicklist-hide-first' }}
	>
		<Switch
			id="quicklist-hide-first"
			bind:checked={config.quicklist_hide_first}
			onCheckedChange={() => void updateSetting('quicklist_hide_first')}
		/>
	</CommonSetting>
	<CommonSetting
		title="Quicklist price strategy"
		description="Choose how the price is calculated from today's median and current in-game sell orders."
	>
		<Button class="text-nowrap" onclick={() => (open = true)}>Set strategy</Button>
	</CommonSetting>
</div>

<Dialog bind:open>
	{#snippet title()}Quicklist price strategy{/snippet}
	{#snippet description()}Preview where your listing appears among five example sell orders.{/snippet}
	<div class="space-y-4 px-6 min-h-0 overflow-y-auto">
		<div class="bg-card/50 p-4 border border-border-secondary text-sm">
			<p class="mb-2 font-semibold">Median: {exampleMedian}p</p>
			{#each preview as offer, index (`${offer.price}-${offer.yours}-${index}`)}
				<div
					class:font-semibold={offer.yours}
					class:text-accent={offer.yours}
					class="py-0.5 tabular-nums"
				>
					{offer.price}p{offer.yours ? ' — your listing' : ''}
				</div>
			{/each}
		</div>
		<fieldset class="space-y-2">
			<legend class="mb-2 font-semibold text-sm">Strategy</legend>
			{#each quicklistStrategies as strategy}
				<label class="flex items-center gap-2 text-sm cursor-pointer">
					<input
						type="radio"
						name="quicklist-strategy"
						checked={config.quicklist_price_strategy === strategy.value}
						onchange={() => selectStrategy(strategy.value)}
					/>
					{strategy.label}
				</label>
			{/each}
		</fieldset>
		<p class="text-muted-foreground text-xs">
			Median prices are rounded to the nearest platinum before the strategy is applied.
		</p>
	</div>
	{#snippet dialogActions()}<Button onclick={() => (open = false)}>Done</Button>{/snippet}
</Dialog>
