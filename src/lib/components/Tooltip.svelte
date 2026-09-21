<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Tooltip } from 'bits-ui';
	import { cn } from '$lib/utils';

	let {
		children,
		content,
		delayDuration = 200,
		side = 'top',
		align = 'center',
		class: className,
	}: {
		children: Snippet;
		content: Snippet;
		delayDuration?: number;
		side?: 'top' | 'right' | 'bottom' | 'left';
		align?: 'start' | 'center' | 'end';
		class?: string;
	} = $props();
</script>

<Tooltip.Provider {delayDuration} skipDelayDuration={100}>
	<Tooltip.Root>
		<Tooltip.Trigger class={cn('focus-visible:outline-2 focus-visible:outline-accent', className)}>
			{@render children()}
		</Tooltip.Trigger>
		<Tooltip.Portal>
			<Tooltip.Content
				{side}
				{align}
				sideOffset={8}
				collisionPadding={12}
				class="z-100 bg-surface p-3 border border-border max-w-80 text-surface-foreground text-sm shadow-xl data-[state=closed]:animate-out data-[state=delayed-open]:animate-in"
			>
				{@render content()}
				<Tooltip.Arrow class="fill-surface" />
			</Tooltip.Content>
		</Tooltip.Portal>
	</Tooltip.Root>
</Tooltip.Provider>
