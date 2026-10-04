<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Tooltip, type WithoutChildren } from 'bits-ui';
	import { cn } from '$lib/utils';

	let {
		children,
		trigger,
		content,
		disabled = false,
		delayDuration = 200,
		side = 'top',
		align = 'center',
		class: className,
		triggerProps,
		triggerTag = 'button',
	}: {
		children?: Snippet;
		trigger?: NonNullable<Tooltip.TriggerProps['child']>;
		content: Snippet;
		disabled?: boolean;
		delayDuration?: number;
		side?: 'top' | 'right' | 'bottom' | 'left';
		align?: 'start' | 'center' | 'end';
		class?: string;
		triggerProps?: WithoutChildren<Tooltip.TriggerProps>;
		triggerTag?: 'button' | 'div';
	} = $props();
</script>

<Tooltip.Provider {delayDuration} skipDelayDuration={100}>
	<Tooltip.Root {disabled}>
		<Tooltip.Trigger
			{...triggerProps}
			class={cn(
				'focus-visible:outline-2 focus-visible:outline-accent',
				className,
				triggerProps?.class,
			)}
		>
			{#snippet child({ props })}
				{#if trigger}
					<!-- Compose with an existing control without adding another trigger element. -->
					{@render trigger({ props })}
				{:else if triggerTag === 'div'}
					<!-- svelte-ignore a11y_no_noninteractive_tabindex (Keyboard users can focus the container to read its tooltip.) -->
					<div {...props} tabindex="0">{@render children?.()}</div>
				{:else}
					<button {...props}>{@render children?.()}</button>
				{/if}
			{/snippet}
		</Tooltip.Trigger>
		<Tooltip.Portal>
			<Tooltip.Content
				{side}
				{align}
				sideOffset={8}
				collisionPadding={12}
				class="z-100 bg-surface shadow-xl max-w-80 text-surface-foreground text-base data-[state=closed]:animate-out data-[state=delayed-open]:animate-in"
			>
				<div class="p-2 border">
					{@render content()}
				</div>
				<!-- <Tooltip.Arrow /> -->
			</Tooltip.Content>
		</Tooltip.Portal>
	</Tooltip.Root>
</Tooltip.Provider>
