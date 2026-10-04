<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Dialog as BitsDialog, type WithoutChild } from 'bits-ui';
	import { cn } from '$lib/utils';
	import { windowDrag } from '$lib/window-drag';

	type Props = BitsDialog.RootProps & {
		trigger?: Snippet;
		title: Snippet;
		description: Snippet;
		dialogClose?: Snippet;
		dialogActions?: Snippet;
		belowContent?: Snippet;
		blurBackdrop?: boolean;
		strongBackdrop?: boolean;
		contentProps?: WithoutChild<BitsDialog.ContentProps>;
	};

	let {
		open = $bindable(false),
		children,
		trigger,
		title,
		description,
		dialogClose,
		dialogActions,
		belowContent,
		blurBackdrop = true,
		strongBackdrop = false,
		contentProps,
		...restProps
	}: Props = $props();
</script>

{#snippet dialogContent()}
	<BitsDialog.Content
		{...contentProps}
		class={cn(
			'gap-4 grid bg-background py-6 border outline-hidden w-[min(42rem,calc(100vw-2rem))] min-w-0 overflow-hidden artus-modal-content',
			belowContent
				? 'relative min-h-0 max-h-full flex-1'
				: 'top-1/2 left-1/2 z-50 fixed h-[min(42rem,calc(100vh-2rem))] -translate-x-1/2 -translate-y-1/2',
			contentProps?.class,
		)}
	>
		<div class="px-6 select-none" use:windowDrag>
			<BitsDialog.Title class="font-expanded font-bold text-xl">
				{@render title()}
			</BitsDialog.Title>
			<BitsDialog.Description class="mt-1 text-muted-foreground text-base">
				{@render description()}
			</BitsDialog.Description>
		</div>

		{@render children?.()}

		{#if dialogClose || dialogActions}
			<div class="flex justify-end gap-2 px-6">
				{#if dialogClose}
					<BitsDialog.Close>
						{@render dialogClose()}
					</BitsDialog.Close>
				{/if}
				{@render dialogActions?.()}
			</div>
		{/if}
	</BitsDialog.Content>
{/snippet}

<BitsDialog.Root bind:open {...restProps}>
	{#if trigger}
		<BitsDialog.Trigger>
			{@render trigger()}
		</BitsDialog.Trigger>
	{/if}
	<BitsDialog.Portal>
		<BitsDialog.Overlay
			class={cn('z-50 fixed inset-0 artus-modal-overlay', strongBackdrop ? 'bg-black/80' : 'bg-black/50', blurBackdrop && 'backdrop-blur-xs')}
		/>
		{#if belowContent}
			<div class="top-1/2 left-1/2 z-50 fixed flex flex-col gap-2 w-[min(42rem,calc(100vw-2rem))] max-h-[calc(100vh-2rem)] -translate-x-1/2 -translate-y-1/2">
				{@render dialogContent()}
				{#if open}{@render belowContent()}{/if}
			</div>
		{:else}
			{@render dialogContent()}
		{/if}
	</BitsDialog.Portal>
</BitsDialog.Root>
