<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Popover } from 'bits-ui';
	import { cn } from '$lib/utils';

	let {
		open = $bindable(false),
		trigger,
		children,
		align = 'end',
		contentClass,
		triggerClass,
		triggerAriaLabel,
	}: {
		open?: boolean;
		trigger: Snippet;
		children: Snippet;
		align?: 'start' | 'center' | 'end';
		contentClass?: string;
		triggerClass?: string;
		triggerAriaLabel: string;
	} = $props();
</script>

<Popover.Root bind:open>
	<Popover.Trigger aria-label={triggerAriaLabel} class={cn('cursor-pointer', triggerClass)}>
		{@render trigger()}
	</Popover.Trigger>
	<Popover.Portal>
		<Popover.Content
			{align}
			sideOffset={8}
			class={cn('z-50 bg-background shadow-xl border border-border-secondary p-1 min-w-44 outline-none', contentClass)}
		>
			{@render children()}
		</Popover.Content>
	</Popover.Portal>
</Popover.Root>
