<script lang="ts">
	import Icon from '@iconify/svelte';
	import { mode } from 'mode-watcher';
	import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
	import Button from '$lib/components/Button.svelte';
	import Combobox from '$lib/components/Combobox.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import Select from '$lib/components/Select.svelte';
	import {
		addMarketNotificationRule,
		loadMarketNotificationItems,
		marketNotificationState,
		removeMarketNotificationRule,
		type MarketNotificationDuration,
		type MarketNotificationIntent,
	} from '$lib/market-notifications.svelte';

	let {
		open = $bindable(false),
		showTrigger = true,
		currentItem,
	}: {
		open?: boolean;
		showTrigger?: boolean;
		currentItem?: { slug: string; name: string };
	} = $props();
	let selectedSlug = $state('');
	let intent = $state<MarketNotificationIntent>('buy');
	let price = $state<number | undefined>(undefined);
	let duration = $state<MarketNotificationDuration>('session');
	let notifyOnce = $state(true);
	let saving = $state(false);
	let saveError = $state<string | null>(null);
	let scrollbarTheme = $derived(mode.current === 'light' ? 'os-theme-dark' : 'os-theme-light');
	let availableItems = $derived.by(() => {
		if (
			!currentItem ||
			marketNotificationState.items.some((item) => item.value === currentItem.slug)
		) {
			return marketNotificationState.items;
		}
		return [{ label: currentItem.name, value: currentItem.slug }, ...marketNotificationState.items];
	});
	let selectedItem = $derived(availableItems.find((item) => item.value === selectedSlug));
	let canAdd = $derived(Boolean(selectedItem && price !== undefined && price > 0 && !saving));

	const intentOptions = [
		{ value: 'buy', label: 'Buy' },
		{ value: 'sell', label: 'Sell' },
	];
	const durationOptions = [
		{ value: 'session', label: 'Until app closes' },
		{ value: '1h', label: '1 hour' },
		{ value: '6h', label: '6 hours' },
		{ value: '24h', label: '24 hours' },
		{ value: '7d', label: '7 days' },
		{ value: 'unlimited', label: 'Unlimited' },
	];

	$effect(() => {
		if (open) void loadMarketNotificationItems().catch(() => undefined);
	});

	async function addRule() {
		if (!canAdd || !selectedItem || price === undefined) return;
		saving = true;
		saveError = null;
		try {
			await addMarketNotificationRule({
				slug: selectedItem.value,
				itemName: selectedItem.label,
				intent,
				price,
				duration,
				notifyOnce,
			});
			selectedSlug = '';
			price = undefined;
		} catch (error) {
			console.error('Could not add market notification rule:', error);
			saveError = 'Could not create the market notification.';
		} finally {
			saving = false;
		}
	}

	function ruleDescription(rule: (typeof marketNotificationState.rules)[number]) {
		return rule.intent === 'sell'
			? `Sell when a buy order reaches at least ${rule.price.toLocaleString()} platinum`
			: `Buy when a sell order reaches at most ${rule.price.toLocaleString()} platinum`;
	}

	function expiryDescription(rule: (typeof marketNotificationState.rules)[number]) {
		if (rule.sessionOnly) return 'Until app closes';
		if (rule.expiresAt === null) return 'Unlimited';
		return `Until ${new Intl.DateTimeFormat(undefined, {
			dateStyle: 'medium',
			timeStyle: 'short',
		}).format(rule.expiresAt)}`;
	}

	function useCurrentItem() {
		if (!currentItem) return;
		selectedSlug = currentItem.slug;
		saveError = null;
	}
</script>

{#snippet trigger()}
	{#if showTrigger}
		<Button title="Configure market notifications" size="icon" class="p-2.5">
			<Icon icon="material-symbols:notification-add-outline-rounded" class="size-5" />
		</Button>
	{/if}
{/snippet}

{#snippet title()}Market notifications{/snippet}

{#snippet description()}
	Watch newly posted Warframe Market orders and notify when your target price is reached.
{/snippet}

{#snippet dialogClose()}<Button>Done</Button>{/snippet}

<Dialog bind:open trigger={showTrigger ? trigger : undefined} {title} {description} {dialogClose}>
	<OverlayScrollbarsComponent
		defer
		class="flex-1 mr-1.75 min-w-0 min-h-0"
		options={{ scrollbars: { theme: scrollbarTheme, autoHide: 'move' } }}
	>
		<div class="flex flex-col gap-5 pr-4.25 pl-6">
			<section aria-labelledby="new-market-notification-heading">
				<div class="flex justify-between items-baseline mb-2">
					<h3 id="new-market-notification-heading" class="font-medium text-sm">New notification</h3>
					<span
						class:text-success={marketNotificationState.connection === 'connected'}
						class:text-warn={marketNotificationState.connection === 'connecting'}
						class="text-muted-foreground text-xs capitalize"
					>
						Live feed: {marketNotificationState.connection}
					</span>
				</div>
				<div class="flex flex-col gap-4 p-4 border">
					<div>
						<div class="flex justify-between items-center gap-3 mb-1.5">
							<label
								for="market-notification-item"
								class="font-semibold text-muted-foreground text-xs"
							>
								Item
							</label>
							{#if currentItem}
								<button
									type="button"
									onclick={useCurrentItem}
									class="flex items-center gap-1 text-muted-foreground hover:text-foreground text-xs cursor-pointer"
									title={`Select ${currentItem.name}`}
								>
									<Icon icon="material-symbols:subdirectory-arrow-left-rounded" class="size-3.5" />
									Use current item
								</button>
							{/if}
						</div>
						<Combobox
							type="single"
							items={availableItems}
							bind:value={selectedSlug}
							inputValue={selectedItem?.label ?? ''}
							disabled={marketNotificationState.itemsLoading}
							inputProps={{
								id: 'market-notification-item',
								placeholder: marketNotificationState.itemsLoading
									? 'Loading items...'
									: 'Search for an item...',
							}}
						/>
					</div>
					<div class="items-start gap-4 grid grid-cols-1 sm:grid-cols-3">
						<div>
							<div class="mb-1.5 font-semibold text-muted-foreground text-xs">I want to</div>
							<RadioGroup
								label="Market notification intent"
								options={intentOptions}
								bind:value={intent}
							/>
						</div>
						<div>
							<label
								for="market-notification-price"
								class="block mb-1.5 font-semibold text-muted-foreground text-xs"
							>
								{intent === 'sell' ? 'Minimum price' : 'Maximum price'}
							</label>
							<div class="flex items-center border w-full h-10">
								<input
									id="market-notification-price"
									type="number"
									min="1"
									step="1"
									bind:value={price}
									placeholder="Platinum"
									class="bg-transparent px-3 outline-none w-full h-full text-sm"
								/>
								<img src="/icons/platinum.png" alt="Platinum" class="mr-3 size-4" />
							</div>
						</div>
						<div>
							<label
								for="market-notification-duration"
								class="block mb-1.5 font-semibold text-muted-foreground text-xs"
							>
								Duration
							</label>
							<Select
								type="single"
								items={durationOptions}
								bind:value={duration}
								triggerProps={{ id: 'market-notification-duration', class: 'max-w-none h-10' }}
							/>
						</div>
					</div>
					<div class="flex flex-col gap-3 pt-3 border-t">
						<label class="flex items-start gap-2 cursor-pointer select-none">
							<input
								type="checkbox"
								bind:checked={notifyOnce}
								class="mt-0.5 size-4 accent-accent cursor-pointer"
							/>
							<span>
								<span class="block font-medium text-sm">Stop after the first match</span>
								<span class="block text-muted-foreground text-xs">
									The rule is removed after sending one notification.
								</span>
							</span>
						</label>
						<div class="flex items-center gap-2 text-muted-foreground text-xs">
							<Icon icon="material-symbols:filter-alt-outline-rounded" class="size-4 shrink-0" />
							<span>
								{intent === 'sell'
									? 'Watching new buy orders at or above your target.'
									: 'Watching new sell orders at or below your target.'}
							</span>
						</div>
					</div>
					{#if marketNotificationState.itemsError}
						<p role="alert" class="text-danger text-xs">{marketNotificationState.itemsError}</p>
					{/if}
					{#if saveError}<p role="alert" class="text-danger text-xs">{saveError}</p>{/if}
					<div class="flex justify-end">
						<Button
							class="min-w-36"
							variant="primary"
							disabled={!canAdd}
							onclick={() => void addRule()}
						>
							{saving ? 'Adding...' : 'Add notification'}
						</Button>
					</div>
				</div>
			</section>

			<section aria-labelledby="active-market-notifications-heading">
				<div class="flex justify-between items-baseline mb-2">
					<h3 id="active-market-notifications-heading" class="font-medium text-sm">
						Active notifications
					</h3>
					<span class="text-muted-foreground text-xs">
						{marketNotificationState.rules.length} active
					</span>
				</div>
				<div class="flex flex-col gap-2">
					{#each marketNotificationState.rules as rule (rule.id)}
						<article class="flex justify-between items-center gap-3 p-3 border">
							<div class="min-w-0">
								<h4 class="font-medium text-sm truncate">{rule.itemName}</h4>
								<p class="text-muted-foreground text-xs">{ruleDescription(rule)}</p>
								<p class="mt-1 text-[11px] text-muted-foreground">
									{expiryDescription(rule)}{rule.notifyOnce !== false ? ' • First match only' : ''}
								</p>
							</div>
							<button
								type="button"
								onclick={() => void removeMarketNotificationRule(rule.id)}
								aria-label={`Remove ${rule.itemName} notification`}
								class="hover:bg-danger/15 p-2 hover:text-danger cursor-pointer shrink-0"
							>
								<Icon icon="material-symbols:delete-outline-rounded" class="size-4" />
							</button>
						</article>
					{:else}
						<p class="p-4 border text-muted-foreground text-sm">No active market notifications.</p>
					{/each}
				</div>
				{#if marketNotificationState.connectionError}
					<p role="status" class="mt-2 text-warn text-xs">
						{marketNotificationState.connectionError} Reconnecting automatically.
					</p>
				{/if}
			</section>
		</div>
	</OverlayScrollbarsComponent>
</Dialog>
