<script lang="ts">
	import Icon from '@iconify/svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Button } from 'bits-ui';
	import { appNavigation, navigateBack, navigateForward } from '$lib/app-navigation.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
	import { windowDrag } from '$lib/window-drag';
	import NotificationCenter from './NotificationCenter.svelte';
	import MarketProfile from './MarketProfile.svelte';

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
		subtitle,
		onOpenNotificationSettings,
	}: { title?: string; subtitle?: string; onOpenNotificationSettings?: () => void } = $props();
</script>

<header class="flex justify-between items-center w-full" use:windowDrag>
	<!-- Left title -->
	<div class="flex items-center select-none">
		<div class="flex items-center w-48 shrink-0">
			<div class="flex items-center pl-1.5">
				<Button.Root
					aria-label="Go back"
					title="Back"
					onclick={navigateBack}
					disabled={appNavigation.index === 0}
					class="hover:bg-elevated disabled:opacity-40 p-0.75 rounded cursor-pointer disabled:cursor-default"
				>
					<Icon icon="material-symbols:arrow-back-rounded" class="size-5" />
				</Button.Root>
				<Button.Root
					aria-label="Go forward"
					title="Forward"
					onclick={navigateForward}
					disabled={appNavigation.index === appNavigation.entries.length - 1}
					class="hover:bg-elevated disabled:opacity-40 p-0.75 rounded cursor-pointer disabled:cursor-default"
				>
					<Icon icon="material-symbols:arrow-forward-rounded" class="size-5" />
				</Button.Root>
			</div>
			<div class="flex-1 pr-1 font-expanded font-black text-accent text-sm text-center uppercase">
				Artus
			</div>
		</div>
		<div class="bg-muted -ml-px rounded-full w-0.5 h-4 text-sm rotate-12"></div>
		<div class="px-4">{title}</div>
		{#if subtitle}
			<div class="bg-muted -ml-px rounded-full w-0.5 h-4 text-sm rotate-12"></div>
			<div class="px-4">{subtitle}</div>
		{/if}
	</div>
	<!-- Right controls -->
	<div class="flex items-center gap-2">
		<Button.Root
			aria-label="Full-width content"
			aria-pressed={config.full_width_content}
			title={config.full_width_content ? 'Use centered content' : 'Use full-width content'}
			class="hover:bg-elevated p-1 rounded focus-visible:outline-2 focus-visible:outline-accent cursor-pointer"
			onclick={() => {
				config.full_width_content = !config.full_width_content;
				void updateSetting('full_width_content');
			}}
		>
			<Icon
				icon={config.full_width_content ? 'lucide:minimize-2' : 'lucide:maximize-2'}
				class="size-5"
			/>
		</Button.Root>
		{#if !config.hide_donate_button}
			<Button.Root
				href="https://ko-fi.com/thaumictom"
				target="_blank"
				class="flex items-center gap-2 hover:bg-elevated px-2 py-1 border text-base"
			>
				<Icon icon="simple-icons:kofi" class="size-4" />
				Donate
			</Button.Root>
		{/if}
		<NotificationCenter {onOpenNotificationSettings} />
		<MarketProfile />
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
