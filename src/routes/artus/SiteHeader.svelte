<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';

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

<header class="flex justify-between items-center pl-4 border-b w-full" data-tauri-drag-region>
	<div class="flex items-center select-none">
		<div class="font-expanded font-black text-secondary text-xs uppercase">Artus</div>
		<div class="bg-secondary mx-4 w-px h-3 text-xs"></div>
		<div>{title}</div>
	</div>
	<div class="flex *:px-4 *:h-10 overflow-hidden">
		<Button variant="ghost" onclick={minimize} aria-label="Minimize window" class="border-0!">
			<Icon icon="mdi:minimize" aria-hidden="true" />
		</Button>
		<Button
			variant="ghost"
			onclick={toggleMaximize}
			aria-label="Toggle maximize window"
			class="border-0!"
		>
			{#if isMaximized}
				<Icon icon="mdi:window-restore" class="text-xs" aria-hidden="true" />
			{:else}
				<Icon icon="mdi:window-maximize" aria-hidden="true" />
			{/if}
		</Button>
		<Button
			variant="ghost"
			onclick={closeWindow}
			aria-label="Close window"
			class="hover:bg-destructive/50! border-0!"
		>
			<Icon icon="mdi:window-close" aria-hidden="true" />
		</Button>
	</div>
</header>
