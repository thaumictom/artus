<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Reward } from 'warframe-worldstate-parser';
	import Checkbox from './Checkbox.svelte';
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
			<label
				class="inline-flex shrink-0 items-center gap-2 rounded-full border px-2.5 py-1 text-sm font-medium transition-colors {completed
					? 'border-accent/30 bg-accent/10 text-accent hover:bg-accent/15'
					: 'border-border-secondary bg-surface/30 text-muted-foreground hover:bg-surface hover:text-foreground'} {disabled ? 'cursor-not-allowed' : 'cursor-pointer'}"
			>
				<Checkbox
					checked={completed}
					class="data-[state=checked]:bg-transparent data-[state=checked]:border-accent/40 rounded-sm size-4 data-[state=checked]:text-accent"
					{disabled}
					aria-label={`Mark ${node} as ${completed ? 'incomplete' : 'completed'}`}
					onCheckedChange={(checked) => onCompletedChange?.(checked)}
				/>
				<span class="sr-only sm:not-sr-only">{completed ? 'Completed' : 'Complete'}</span>
			</label>
		{/if}
	{/snippet}
	{#snippet footer()}
		<h3 class="font-medium text-sm break-words">{node}</h3>
		{#if details}
			<p class="mt-1 text-muted-foreground text-sm break-words">{@render details()}</p>
		{/if}
	{/snippet}
</ViewCard>
