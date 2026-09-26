<script lang="ts">
	import { onDestroy } from 'svelte';
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

	let outsideAttempt = $state(false);
	let outsideAttemptTimer: ReturnType<typeof setTimeout> | undefined;
	let outsideAttemptFrame: number | undefined;

	function notifyOutsideAttempt() {
		outsideAttempt = false;
		if (outsideAttemptTimer) clearTimeout(outsideAttemptTimer);
		if (outsideAttemptFrame !== undefined) cancelAnimationFrame(outsideAttemptFrame);
		outsideAttemptFrame = requestAnimationFrame(() => {
			outsideAttemptFrame = undefined;
			outsideAttempt = true;
			outsideAttemptTimer = setTimeout(() => {
				outsideAttempt = false;
				outsideAttemptTimer = undefined;
			}, 500);
		});
	}

	function dismissOnEscape(event: KeyboardEvent) {
		contentProps?.onEscapeKeydown?.(event);
		open = false;
		event.preventDefault();
	}

	onDestroy(() => {
		if (outsideAttemptTimer) clearTimeout(outsideAttemptTimer);
		if (outsideAttemptFrame !== undefined) cancelAnimationFrame(outsideAttemptFrame);
	});
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
			onpointerdown={notifyOutsideAttempt}
			class={cn(
				'z-50 fixed inset-0 bg-black/50 data-[state=open]:backdrop-blur-xs data-[state=closed]:animate-out data-[state=open]:animate-in artus-modal-overlay artus-alert-dialog-overlay data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0',
				outsideAttempt && 'alert-outside-attempt',
			)}
		/>
		<AlertDialog.Content
			{...contentProps}
			onEscapeKeydown={dismissOnEscape}
			class={cn(
				'top-1/2 left-1/2 z-50 fixed gap-2 grid bg-background p-7 border outline-hidden max-w-lg -translate-x-1/2 -translate-y-1/2 data-[state=closed]:animate-out data-[state=open]:animate-in alert-dialog-content data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95',
				outsideAttempt && 'alert-outside-attempt',
				contentProps?.class,
			)}
		>
			<AlertDialog.Title class="font-expanded font-bold text-lg">
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
