<script lang="ts">
	import Icon from '@iconify/svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Button } from 'bits-ui';

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

	let { title = 'Artus' } = $props();
</script>

<header class="flex justify-between items-center w-full" data-tauri-drag-region>
	<!-- Left title -->
	<div class="flex items-center select-none">
		<div class="px-4 font-expanded font-black text-accent text-xs uppercase">Artus</div>
		<div class="bg-muted w-px h-3 text-xs"></div>
		<div class="px-4">{title}</div>
	</div>
	<!-- Right controls -->
	<div class="flex *:hover:bg-elevated *:px-4 *:h-10 overflow-hidden *:cursor-pointer">
		<Button.Root aria-label="Minimize window" onclick={minimize} tabindex={-1}>
			<Icon icon="mdi:minimize" />
		</Button.Root>
		<Button.Root onclick={toggleMaximize} aria-label="Toggle maximize window" tabindex={-1}>
			{#if isMaximized}
				<Icon icon="mdi:window-restore" class="text-xs" />
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
</header>
