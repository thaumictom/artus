<script>
	import { Separator } from 'bits-ui';
	import { toKebabCase } from '@std/text';
	import WarframeSettings from './categories/WarframeSettings.svelte';
	import Hotkeys from './categories/Hotkeys.svelte';
	import MetaInformation from './categories/MetaInformation.svelte';

	let components = [
		{ name: 'Hotkeys', component: Hotkeys },
		{ name: 'Warframe Settings', component: WarframeSettings },
	];
</script>

<div class="@container flex">
	<!-- Categories -->
	<div class="flex-1">
		{#each components as { name, component: Component }, i}
			<h1 class="mb-4 font-black text-lg scroll-mt-4" id={toKebabCase(name)}>
				{name}
			</h1>
			<Component />
			<Separator.Root class="my-12 bg-border h-px" />
		{/each}
		<MetaInformation />
	</div>
	<!-- Table of contents -->
	<div class="top-4 sticky ml-4 pl-4 border-border border-l max-w-40 h-min">
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
