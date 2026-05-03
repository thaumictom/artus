<script lang="ts">
	import type { Snippet } from 'svelte';
	import { AlertDialog, type WithoutChild } from 'bits-ui';
	import Button from './Button.svelte';
	import { cn } from '$lib/utils';

	type Props = AlertDialog.RootProps & {
		buttonText?: string;
		title: Snippet;
		description: Snippet;
		dialogCancel: Snippet;
		dialogAction: Snippet;
		contentProps?: WithoutChild<AlertDialog.ContentProps>;
	};

	let {
		open = $bindable(false),
		children,
		buttonText,
		contentProps,
		title,
		description,
		dialogCancel,
		dialogAction,
		...restProps
	}: Props = $props();
</script>

<AlertDialog.Root bind:open {...restProps}>
	{#if buttonText}
		<AlertDialog.Trigger>
			<Button>
				{buttonText}
			</Button>
		</AlertDialog.Trigger>
	{/if}
	<AlertDialog.Portal>
		<AlertDialog.Overlay
			class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 fixed inset-0 z-50 bg-black/50 data-[state=open]:backdrop-blur-xs"
		/>
		<AlertDialog.Content
			class={cn(
				'bg-background data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 outline-hidden fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 grid max-w-lg gap-2 border p-7',
				contentProps?.class,
			)}
			{...contentProps}
		>
			<AlertDialog.Title class="text-lg font-bold font-expanded">
				{@render title()}
			</AlertDialog.Title>
			<AlertDialog.Description>
				{@render description()}
			</AlertDialog.Description>
			{@render children?.()}
			<div class="flex justify-end gap-2 mt-6">
				<AlertDialog.Cancel children={dialogCancel} />
				<AlertDialog.Action children={dialogAction} />
			</div>
		</AlertDialog.Content>
	</AlertDialog.Portal>
</AlertDialog.Root>
