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
	<b class="text-sm">{title}</b>
	<div class="flex overflow-hidden">
		<Button variant="ghost" onclick={minimize} aria-label="Minimize window" class="border-0!">
			<Icon icon="lucide:minus" aria-hidden="true" />
		</Button>
		<Button
			variant="ghost"
			onclick={toggleMaximize}
			aria-label="Toggle maximize window"
			class="border-0!"
		>
			{#if isMaximized}
				<Icon icon="lucide:minimize" class="text-xs" aria-hidden="true" />
			{:else}
				<Icon icon="lucide:maximize" aria-hidden="true" />
			{/if}
		</Button>
		<Button
			variant="ghost"
			onclick={closeWindow}
			aria-label="Close window"
			class="hover:bg-destructive/50! border-0!"
		>
			<Icon icon="lucide:x" aria-hidden="true" />
		</Button>
	</div>
</header>
