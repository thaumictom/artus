<script lang="ts">
	import { Button, Tabs } from 'bits-ui';
	import Icon from '@iconify/svelte';
	import type { Sections } from '$lib/types';

	let {
		sections,
	}: {
		sections: Sections;
	} = $props();

	let isSidebarOpen = $state(true);
</script>

<Tabs.List
	class="flex flex-col justify-between bg-surface px-2 pb-1 h-full text-surface-foreground"
>
	<div class="flex flex-col">
		{#each Object.entries(sections) as [id, section]}
			<Tabs.Trigger value={id} class="data-[state=active]:bg-elevated p-1 rounded">
				<div class="flex items-center h-full">
					<Icon icon={section.icon} class="size-6" />
					<span
						class={{
							' overflow-hidden text-sm whitespace-nowrap transition-all duration-300 ease-in-out': true,
							'opacity-100 max-w-20': isSidebarOpen,
							'opacity-0 max-w-0': !isSidebarOpen,
						}}
						aria-hidden={!isSidebarOpen}
					>
						<span class="pr-1 pl-2">
							{section.label}
						</span>
					</span>
				</div>
			</Tabs.Trigger>
		{/each}
	</div>
	<div>
		<Button.Root
			class="hover:bg-elevated p-1 rounded"
			aria-label="Toggle sidebar"
			onclick={() => (isSidebarOpen = !isSidebarOpen)}
		>
			{#if isSidebarOpen}
				<Icon icon="material-symbols:left-panel-close-outline-rounded" class="size-6" />
			{:else}
				<Icon icon="material-symbols:left-panel-open-outline-rounded" class="size-6" />
			{/if}
		</Button.Root>
	</div>
</Tabs.List>
