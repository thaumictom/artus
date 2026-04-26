<script>
	import { Separator } from 'bits-ui';
	import { toKebabCase } from '@std/text';
	import WarframeSettings from './categories/WarframeSettings.svelte';
	import Hotkeys from './categories/Hotkeys.svelte';
	import MetaInformation from './categories/MetaInformation.svelte';
	import OverlaySettings from './categories/OverlaySettings.svelte';

	let components = [
		{ name: 'Hotkeys', component: Hotkeys },
		{ name: 'Warframe Settings', component: WarframeSettings },
		{ name: 'Overlay Settings', component: OverlaySettings },
	];
</script>

<div class="flex mx-auto p-8 w-full max-w-2xl">
	<!-- Categories -->
	<div class="flex-1">
		{#each components as { name, component: Component }, i}
			<h1 class="mb-8 font-bold text-lg scroll-mt-8" id={toKebabCase(name)}>
				{name}
			</h1>
			<Component />
			<Separator.Root class="my-8 bg-border h-px" />
		{/each}
		<MetaInformation />
	</div>
	<!-- Table of contents -->
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
