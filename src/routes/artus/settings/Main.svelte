<script lang="ts">
	import { Separator } from 'bits-ui';
	import WarframeSettings from './categories/WarframeSettings.svelte';
	import Hotkeys from './categories/Hotkeys.svelte';
	import MetaInformation from './categories/MetaInformation.svelte';
	import OverlaySettings from './categories/OverlaySettings.svelte';
	import DebugSettings from './categories/DebugSettings.svelte';
	import { onMount } from 'svelte';
	import { load, Store, LazyStore } from '@tauri-apps/plugin-store';
	import { loadSettings } from '$lib/settings.svelte';
	import { kebabCase } from 'change-case';

	let components = [
		{ name: 'Hotkeys', component: Hotkeys },
		{ name: 'Warframe Settings', component: WarframeSettings },
		{ name: 'Overlay Settings', component: OverlaySettings },
		{ name: 'Debug', component: DebugSettings },
	];

	onMount(() => {
		loadSettings();
	});
</script>

<div class="flex mx-auto p-8 w-full max-w-2xl">
	<div class="flex-1">
		{#each components as { name, component: Component }, i}
			<h1 class="mb-8 font-bold text-lg scroll-mt-8" id={kebabCase(name)}>
				{name}
			</h1>
			<Component />
			<Separator.Root class="my-8 bg-border h-px" />
		{/each}
		<MetaInformation />
	</div>
	<div class="top-8 sticky ml-4 pl-4 border-border border-l h-min">
		<p class="mb-2 font-medium text-sm">Contents</p>
		<ul class="text-muted-foreground text-sm">
			{#each components as { name }, i}
				<li class="mb-1">
					<a href={'#' + name.toLowerCase().replace(/\s+/g, '-')} class="hover:underline">
						{name}
					</a>
				</li>
			{/each}
		</ul>
	</div>
</div>
