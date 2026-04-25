<script>
	import { Separator } from 'bits-ui';
	import { toKebabCase } from '@std/text';
	import WarframeSettings from './categories/WarframeSettings.svelte';
	import Hotkeys from './categories/Hotkeys.svelte';

	let components = [
		{ name: 'Hotkeys', component: Hotkeys },
		{ name: 'Warframe Settings', component: WarframeSettings },
	];
</script>

<div class="flex">
	<!-- Categories -->
	<div class="flex-1">
		{#each components as { name, component: Component }, i}
			<p class="mb-4 font-expanded font-black" id={toKebabCase(name)}>{name}</p>
			<Component />
			<Separator.Root class="my-4 bg-border h-px" />
		{/each}
		<div class="flex justify-between gap-2">
			<div class="flex-1">
				<p class="text-sm">Build Commit</p>
				<p class="font-mono text-muted-foreground text-xs break-all">7c851af</p>
			</div>
			<div class="flex-1">
				<p class="text-sm">App version</p>
				<p class="font-mono text-muted-foreground text-xs break-all">v0.1.0</p>
			</div>
		</div>
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
