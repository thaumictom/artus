<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Dialog as BitsDialog, type WithoutChild } from 'bits-ui';
	import { cn } from '$lib/utils';

	type Props = BitsDialog.RootProps & {
		trigger?: Snippet;
		title: Snippet;
		description: Snippet;
		dialogClose?: Snippet;
		contentProps?: WithoutChild<BitsDialog.ContentProps>;
	};

	let {
		open = $bindable(false),
		children,
		trigger,
		title,
		description,
		dialogClose,
		contentProps,
		...restProps
	}: Props = $props();
</script>

<BitsDialog.Root bind:open {...restProps}>
	{#if trigger}
		<BitsDialog.Trigger>
			{@render trigger()}
		</BitsDialog.Trigger>
	{/if}
	<BitsDialog.Portal>
		<BitsDialog.Overlay
			class="z-50 fixed inset-0 bg-black/50 data-[state=open]:backdrop-blur-xs data-[state=closed]:animate-out data-[state=open]:animate-in data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
		/>
		<BitsDialog.Content
			{...contentProps}
			class={cn(
				'top-1/2 left-1/2 z-50 fixed gap-2 grid bg-background py-6 border outline-hidden w-[min(42rem,calc(100vw-2rem))] h-[min(42rem,calc(100vh-2rem))] min-w-0 overflow-hidden -translate-x-1/2 -translate-y-1/2 data-[state=closed]:animate-out data-[state=open]:animate-in data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95',
				contentProps?.class,
			)}
		>
			<div class="px-6">
				<BitsDialog.Title class="font-expanded font-bold text-lg">
					{@render title()}
				</BitsDialog.Title>
				<BitsDialog.Description class="mt-1 text-muted-foreground text-sm">
					{@render description()}
				</BitsDialog.Description>
			</div>

			{@render children?.()}

			{#if dialogClose}
				<div class="flex justify-end px-6">
					<BitsDialog.Close>
						{@render dialogClose()}
					</BitsDialog.Close>
				</div>
			{/if}
		</BitsDialog.Content>
	</BitsDialog.Portal>
</BitsDialog.Root>
