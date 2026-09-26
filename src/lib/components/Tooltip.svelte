<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Tooltip, type WithoutChildren } from 'bits-ui';
	import { cn } from '$lib/utils';
	import { fly, fade } from 'svelte/transition';

	let {
		children,
		content,
		delayDuration = 200,
		side = 'top',
		align = 'center',
		class: className,
		triggerProps,
	}: {
		children: Snippet;
		content: Snippet;
		delayDuration?: number;
		side?: 'top' | 'right' | 'bottom' | 'left';
		align?: 'start' | 'center' | 'end';
		class?: string;
		triggerProps?: WithoutChildren<Tooltip.TriggerProps>;
	} = $props();
</script>

<Tooltip.Provider {delayDuration} skipDelayDuration={100}>
	<Tooltip.Root>
		<Tooltip.Trigger
			{...triggerProps}
			class={cn(
				'focus-visible:outline-2 focus-visible:outline-accent',
				className,
				triggerProps?.class,
			)}
		>
			{@render children()}
		</Tooltip.Trigger>
		<Tooltip.Portal>
			<Tooltip.Content
				{side}
				{align}
				sideOffset={8}
				collisionPadding={12}
				class="z-100 bg-surface shadow-xl max-w-80 text-surface-foreground text-sm data-[state=closed]:animate-out data-[state=delayed-open]:animate-in"
			>
				<div class="p-2 border">
					{@render content()}
				</div>
				<Tooltip.Arrow class="text-border" />
			</Tooltip.Content>
		</Tooltip.Portal>
	</Tooltip.Root>
</Tooltip.Provider>
