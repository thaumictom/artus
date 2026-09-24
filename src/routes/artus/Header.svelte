<script lang="ts">
	import Icon from '@iconify/svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Button } from 'bits-ui';
	import { appNavigation, navigateBack, navigateForward } from '$lib/app-navigation.svelte';
	import NotificationCenter from './NotificationCenter.svelte';

	let isMaximized = $state(false);

	const minimize = () => getCurrentWindow().minimize();
	const toggleMaximize = () => getCurrentWindow().toggleMaximize();
	const closeWindow = () => getCurrentWindow().close();

	$effect(() => {
		const appWindow = getCurrentWindow();

		// Explicitly type the unlisten function
		let unlisten: () => void;

		appWindow.isMaximized().then((maximized) => {
			isMaximized = maximized;
		});

		appWindow
			.onResized(async () => {
				isMaximized = await appWindow.isMaximized();
			})
			.then((fn) => (unlisten = fn));

		return () => {
			if (unlisten) unlisten();
		};
	});

	let {
		title = 'Artus',
		onOpenNotificationSettings,
	}: { title?: string; onOpenNotificationSettings?: () => void } = $props();
</script>

<header class="flex justify-between items-center w-full" data-tauri-drag-region>
	<!-- Left title -->
	<div class="flex items-center select-none">
		<div class="flex w-44 shrink-0 items-center">
			<div class="flex items-center gap-1 px-2">
				<Button.Root
					aria-label="Go back"
					title="Back"
					onclick={navigateBack}
					disabled={appNavigation.index === 0}
					class="hover:bg-elevated disabled:opacity-40 p-1 rounded disabled:cursor-default cursor-pointer"
				>
					<Icon icon="material-symbols:arrow-back-rounded" class="size-5" />
				</Button.Root>
				<Button.Root
					aria-label="Go forward"
					title="Forward"
					onclick={navigateForward}
					disabled={appNavigation.index === appNavigation.entries.length - 1}
					class="hover:bg-elevated disabled:opacity-40 p-1 rounded disabled:cursor-default cursor-pointer"
				>
					<Icon icon="material-symbols:arrow-forward-rounded" class="size-5" />
				</Button.Root>
			</div>
			<div class="px-4 font-expanded font-black text-accent text-xs text-center uppercase">Artus</div>
		</div>
		<div class="bg-muted rounded-full w-0.5 h-4 text-xs"></div>
		<div class="px-4">{title}</div>
	</div>
	<!-- Right controls -->
	<div class="flex items-center gap-2">
		<Button.Root
			href="https://ko-fi.com/thaumictom"
			target="_blank"
			class="flex items-center gap-2 hover:bg-elevated px-2 py-1 border text-sm"
		>
			<Icon icon="simple-icons:kofi" class="size-4" />
			Donate
		</Button.Root>
		<NotificationCenter {onOpenNotificationSettings} />
		<div class="flex *:hover:bg-elevated *:px-4 *:h-10 overflow-hidden *:cursor-pointer">
			<Button.Root aria-label="Minimize window" onclick={minimize} tabindex={-1}>
				<Icon icon="mdi:minimize" />
			</Button.Root>
			<Button.Root onclick={toggleMaximize} aria-label="Toggle maximize window" tabindex={-1}>
				{#if isMaximized}
					<Icon icon="mdi:window-restore" />
				{:else}
					<Icon icon="mdi:window-maximize" />
				{/if}
			</Button.Root>
			<Button.Root
				onclick={closeWindow}
				aria-label="Close window"
				class="hover:bg-danger! hover:text-danger-foreground"
				tabindex={-1}
			>
				<Icon icon="mdi:window-close" />
			</Button.Root>
		</div>
	</div>
</header>
