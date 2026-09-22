<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Popover } from 'bits-ui';
	import { tick } from 'svelte';
	import { dashboard } from '$lib/worldstate.svelte';
	import { activeMarketNotificationCount } from '$lib/market-notifications.svelte';
	import { openMarketNotificationTarget } from '$lib/market-navigation.svelte';
	import {
		clearNotificationHistory,
		markAllNotificationsRead,
		notificationCenter,
		type NotificationSource,
	} from '$lib/notifications.svelte';
	import NotificationRuleSettings from './dashboard/widgets/NotificationRuleSettings.svelte';
	import MarketNotificationRules from './market/MarketNotificationRules.svelte';

	let { onOpenNotificationSettings }: { onOpenNotificationSettings?: () => void } = $props();
	let open = $state(false);
	let notificationRulesOpen = $state(false);
	let marketNotificationRulesOpen = $state(false);
	let unreadCount = $derived(notificationCenter.entries.filter((entry) => !entry.read).length);
	let activeMarketRules = $derived(activeMarketNotificationCount());

	function sourceIcon(source: NotificationSource) {
		switch (source) {
			case 'fissure':
				return 'material-symbols:blur-circular-rounded';
			case 'alert':
				return 'material-symbols:warning-outline-rounded';
			case 'invasion':
				return 'material-symbols:swords-outline-rounded';
			case 'dailyDeal':
				return 'material-symbols:local-offer-outline-rounded';
			case 'baro':
				return 'material-symbols:storefront-outline-rounded';
			case 'market':
				return 'material-symbols:shopping-cart-outline-rounded';
		}
	}

	function formatCreatedAt(timestamp: number) {
		return new Intl.DateTimeFormat(undefined, {
			dateStyle: 'medium',
			timeStyle: 'short',
		}).format(timestamp);
	}

	function handleOpenChange(nextOpen: boolean) {
		open = nextOpen;
		if (nextOpen) void markAllNotificationsRead();
	}

	function openNotificationSettings() {
		open = false;
		onOpenNotificationSettings?.();
	}

	async function openNotificationRules() {
		open = false;
		// Let the popover finish restoring focus before opening the dialog. If both
		// happen in the same event, the popover's close cycle dismisses the dialog.
		await tick();
		requestAnimationFrame(() => (notificationRulesOpen = true));
	}

	async function openMarketNotificationRules() {
		open = false;
		await tick();
		requestAnimationFrame(() => (marketNotificationRulesOpen = true));
	}

	function openMarketNotification(entry: (typeof notificationCenter.entries)[number]) {
		if (!entry.market) return;
		open = false;
		openMarketNotificationTarget(
			entry.market.slug,
			entry.market.since ?? entry.createdAt,
			entry.market.orderType,
		);
	}

	function eraLabelClass(era: string) {
		const base = 'border px-1.5 py-0.5 font-medium text-[10px] leading-none shrink-0';
		switch (era.toLowerCase()) {
			case 'lith':
				return `${base} bg-[#d08770]/20 border-[#d08770]/50 text-[#d08770]`;
			case 'meso':
				return `${base} bg-[#4c588a] border-[#7b88a1] text-[#eceff4]`;
			case 'neo':
				return `${base} bg-[#d8dee9]/15 border-[#d8dee9]/50 text-[#d8dee9]`;
			case 'axi':
				return `${base} bg-[#ebcb8b]/20 border-[#ebcb8b]/50 text-[#ebcb8b]`;
			case 'requiem':
				return `${base} bg-[#bf616a]/20 border-[#bf616a]/50 text-[#bf616a]`;
			case 'omnia':
				return `${base} omnia-era border-white/40 text-white`;
			default:
				return `${base} bg-surface text-muted-foreground`;
		}
	}
</script>

<Popover.Root {open} onOpenChange={handleOpenChange}>
	<Popover.Trigger
		aria-label={`Notifications${unreadCount > 0 ? ` (${unreadCount} unread)` : ''}`}
		title="Notifications"
		class="flex justify-center items-center gap-1.5 hover:bg-elevated px-1.5 border min-w-7.5 h-7.5 cursor-pointer"
	>
		<Icon icon="material-symbols:notifications-outline-rounded" class="size-4.5" />
		{#if unreadCount > 0}
			<span
				class="flex justify-center items-center bg-accent px-1 rounded-full min-w-4 h-4 font-bold text-[10px] leading-none text-accent-foreground"
			>
				{unreadCount > 99 ? '99+' : unreadCount}
			</span>
		{/if}
	</Popover.Trigger>
	<Popover.Portal>
		<Popover.Content
			align="end"
			sideOffset={8}
			class="z-50 bg-background shadow-xl border w-[min(26rem,calc(100vw-2rem))] max-h-[min(34rem,calc(100vh-4rem))] overflow-hidden"
		>
			<div class="px-4 py-3 border-b">
				<div class="flex justify-between items-start gap-3">
					<div>
						<h2 class="font-semibold text-sm">Notifications</h2>
						<p class="text-muted-foreground text-xs">Recent world-state and market matches</p>
					</div>
					{#if notificationCenter.entries.length > 0}
						<button
							type="button"
							onclick={() => void clearNotificationHistory()}
							class="hover:text-foreground text-muted-foreground text-xs cursor-pointer shrink-0"
						>
							Clear all
						</button>
					{/if}
				</div>
				<div class="flex flex-wrap justify-end items-center gap-x-3 gap-y-2 mt-3">
					<button
						type="button"
						onclick={() => void openNotificationRules()}
						class="flex items-center gap-1 hover:text-foreground text-muted-foreground text-xs cursor-pointer"
					>
						<Icon icon="material-symbols:notification-add-outline-rounded" class="size-3.5" />
						Rules (world)
					</button>
					<button
						type="button"
						onclick={() => void openMarketNotificationRules()}
						class="flex items-center gap-1 hover:text-foreground text-muted-foreground text-xs cursor-pointer"
					>
						<Icon icon="material-symbols:shopping-cart-outline-rounded" class="size-3.5" />
						Rules (market)
						{#if activeMarketRules > 0}
							<span
								class="flex justify-center items-center bg-accent px-1 rounded-full min-w-4 h-4 font-bold text-[10px] leading-none text-accent-foreground"
							>
								{activeMarketRules}
							</span>
						{/if}
					</button>
					<button
						type="button"
						onclick={openNotificationSettings}
						class="flex items-center gap-1 hover:text-foreground text-muted-foreground text-xs cursor-pointer"
					>
						<Icon icon="material-symbols:settings-outline-rounded" class="size-3.5" />
						Settings
					</button>
				</div>
			</div>
			<div class="max-h-[28rem] overflow-y-auto">
				{#each notificationCenter.entries as entry (entry.id)}
					<svelte:element
						this={entry.market ? 'button' : 'article'}
						type={entry.market ? 'button' : undefined}
						role={entry.market ? 'button' : undefined}
						onclick={entry.market ? () => openMarketNotification(entry) : undefined}
						class="flex gap-3 px-4 py-3 border-b last:border-b-0 w-full text-left"
						class:hover:bg-elevated={Boolean(entry.market)}
						class:cursor-pointer={Boolean(entry.market)}
						title={entry.market ? 'Open item in Market' : undefined}
					>
						<div class="flex justify-center items-center bg-surface mt-0.5 size-8 shrink-0">
							<Icon icon={sourceIcon(entry.source)} class="size-4" />
						</div>
						<div class="flex-1 min-w-0">
							<div class="flex items-center gap-1.5">
								{#if entry.era}
									<span class={eraLabelClass(entry.era)}>{entry.era}</span>
								{/if}
								<h3 class="font-medium text-sm">{entry.title}</h3>
							</div>
							<p class="mt-0.5 text-muted-foreground text-xs">{entry.body}</p>
							<time class="block mt-1.5 text-muted-foreground text-[11px]" datetime={new Date(entry.createdAt).toISOString()}>
								{formatCreatedAt(entry.createdAt)}
							</time>
						</div>
						{#if entry.market}
							<Icon icon="material-symbols:arrow-forward-rounded" class="self-center size-4 shrink-0" />
						{/if}
					</svelte:element>
				{:else}
					<div class="flex flex-col items-center px-6 py-10 text-center">
						<Icon icon="material-symbols:notifications-off-outline-rounded" class="mb-3 size-8 text-muted-foreground" />
						<p class="font-medium text-sm">No notifications yet</p>
						<p class="mt-1 text-muted-foreground text-xs">
							Add notification rules from the Dashboard to watch the world state.
						</p>
					</div>
				{/each}
			</div>
		</Popover.Content>
	</Popover.Portal>
</Popover.Root>

<NotificationRuleSettings
	world={dashboard.world}
	showTrigger={false}
	bind:open={notificationRulesOpen}
/>
<MarketNotificationRules showTrigger={false} bind:open={marketNotificationRulesOpen} />

<style>
	.omnia-era {
		background: linear-gradient(100deg, #d0877060, #4c566a60, #d8dee960, #ebcb8b60, #bf616a60);
	}
</style>
