<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Reward } from 'warframe-worldstate-parser';
	import CompletionToggle from './CompletionToggle.svelte';
	import ViewCard from './ViewCard.svelte';
	import WorldStateReward from './WorldStateReward.svelte';

	let {
		node, reward, children, details, action: customAction, footerAside, expiry, now,
		completed = false, disabled = false, onCompletedChange,
	}: {
		node: string;
		reward?: Reward;
		children?: Snippet;
		details?: Snippet;
		action?: Snippet;
		footerAside?: Snippet;
		expiry?: Date;
		now: number;
		completed?: boolean;
		disabled?: boolean;
		onCompletedChange?: (completed: boolean) => void;
	} = $props();
</script>

<ViewCard
	{completed} {disabled} {expiry} {now} {footerAside}
	onActivate={onCompletedChange ? () => onCompletedChange?.(!completed) : undefined}
>
	{#if children}
		{@render children()}
	{:else}
		<WorldStateReward {reward} />
	{/if}
	{#snippet action()}
		{#if customAction}
			{@render customAction()}
		{:else if onCompletedChange}
			<CompletionToggle {completed} {disabled} label={node} {onCompletedChange} />
		{/if}
	{/snippet}
	{#snippet footer()}
		<h3 class="font-medium text-sm break-words">{node}</h3>
		{#if details}
			<p class="mt-1 text-muted-foreground text-sm break-words">{@render details()}</p>
		{/if}
	{/snippet}
</ViewCard>
