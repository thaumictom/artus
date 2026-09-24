<script lang="ts">
	import Icon from '@iconify/svelte';
	import { toast } from 'svelte-sonner';
	import type z from 'zod';
	import type { OrderWithUserSchema } from '$lib/schemas';
	import Tooltip from './Tooltip.svelte';

	type Order = z.infer<typeof OrderWithUserSchema>;
	let { order, itemName, bulkTradable = false, variantProperty }: {
		order: Order;
		itemName: string;
		bulkTradable?: boolean;
		variantProperty?: 'rank' | 'charges' | 'subtype' | 'amberStars' | 'cyanStars';
	} = $props();
	const timestampFormatter = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'medium' });
	const languageNames = new Intl.DisplayNames(undefined, { type: 'language' });
	const platformNames: Record<string, string> = {
		pc: 'PC', ps4: 'PlayStation', ps5: 'PlayStation', xbox: 'Xbox', switch: 'Nintendo Switch', mobile: 'Mobile',
	};
	function formatPlatform(platform: string) { return platformNames[platform.toLowerCase()] ?? platform; }
	function formatLanguage(locale: string) {
		try { return languageNames.of(locale) ?? locale; } catch { return locale; }
	}
	function platformIcon(platform: string) {
		switch (platform.toLowerCase()) {
			case 'pc': return 'material-symbols:computer-outline-rounded';
			case 'mobile': return 'material-symbols:smartphone-outline-rounded';
			default: return 'material-symbols:sports-esports-outline-rounded';
		}
	}
	function formatTimestamp(timestamp: string) {
		const date = new Date(timestamp);
		return Number.isFinite(date.getTime()) ? timestampFormatter.format(date) : 'Unknown';
	}
	function tradeMessage(): string {
		const action = order.type === 'sell' ? 'buy' : 'sell';
		const perTrade = bulkTradable && order.perTrade !== undefined && order.perTrade > 0 ? order.perTrade : 1;
		const quantity = perTrade > 1 ? `x${perTrade} ` : '';
		const variant = variantProperty && order[variantProperty] !== undefined
			? ` (${variantProperty} ${order[variantProperty]})` : '';
		return `/w ${order.user.ingameName} Hi! I want to ${action}: ${quantity}"${itemName}${variant}" for ${order.platinum} platinum. (warframe.market)`;
	}
	async function copyToClipboard() {
		try {
			await navigator.clipboard.writeText(tradeMessage());
			toast.success('Copied to clipboard');
		} catch (err) {
			console.error('Failed to copy:', err);
			toast.error('Failed to copy to clipboard');
		}
	}
</script>

<Tooltip side="left" align="center" class="hover:bg-surface p-1 border cursor-pointer"
	triggerProps={{ 'aria-label': 'Copy trade message to clipboard', onclick: copyToClipboard }}>
	{#snippet children()}<Icon icon="material-symbols:content-copy" class="size-4" />{/snippet}
	{#snippet content()}
		<div class="text-xs whitespace-nowrap">
			<div class="mb-2 font-medium">Copy to clipboard</div>
			<div class="gap-x-3 gap-y-1 grid grid-cols-[auto_1fr]">
				<span class="text-muted-foreground">Platform</span>
				<span class="inline-flex items-center gap-1"><Icon icon={platformIcon(order.user.platform)} class="size-3.5" />{formatPlatform(order.user.platform)}</span>
				<span class="text-muted-foreground">Language</span><span>{formatLanguage(order.user.locale)}</span>
				<span class="text-muted-foreground">Created</span><time datetime={order.createdAt}>{formatTimestamp(order.createdAt)}</time>
				<span class="text-muted-foreground">Updated</span><time datetime={order.updatedAt}>{formatTimestamp(order.updatedAt)}</time>
			</div>
		</div>
	{/snippet}
</Tooltip>
