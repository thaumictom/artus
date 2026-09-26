<script lang="ts">
	import { Button, Tabs, Tooltip } from 'bits-ui';
	import Icon from '@iconify/svelte';
	import type { Sections } from '$lib/types';
	import { marketAccount } from '$lib/market-account.svelte';

	let {
		sections,
	}: {
		sections: Sections;
	} = $props();

	let isSidebarOpen = $state(true);
	let mouseover = $state(false);
</script>

{#snippet navLabel(section: Sections[string])}
	<div class="flex items-center h-full">
		<Icon icon={section.icon} class="size-6" />
		<span
			aria-hidden={!isSidebarOpen}
			class={{
				'overflow-hidden text-sm whitespace-nowrap transition-all duration-300 ease-in-out text-left': true,
				'opacity-100 w-32': isSidebarOpen,
				'opacity-0 w-0': !isSidebarOpen,
			}}
		>
			<span class="pr-1 pl-2">{section.label}</span>
		</span>
	</div>
{/snippet}

<Tabs.List
	class="flex flex-col justify-between bg-surface px-2 pb-1 h-full text-surface-foreground"
	onmouseenter={() => (mouseover = true)}
	onmouseleave={() => (mouseover = false)}
>
	<div class="flex flex-col">
		{#each Object.entries(sections) as [id, section]}
			{#if id === 'listings' && !marketAccount.session}
				<Tooltip.Provider delayDuration={200}>
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<span
									{...props}
									class="block rounded cursor-not-allowed"
									aria-label="Listings unavailable. Log in to warframe.market first."
								>
									<Tabs.Trigger
										value={id}
										disabled
										class="opacity-40 p-1 rounded pointer-events-none"
									>
										{@render navLabel(section)}
									</Tabs.Trigger>
								</span>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Portal>
							<Tooltip.Content
								side="right"
								sideOffset={8}
								collisionPadding={12}
								class="z-100 bg-surface shadow-xl p-3 border border-border max-w-64 text-surface-foreground text-sm"
							>
								Log in to warframe.market first to view your listings.
								<Tooltip.Arrow class="text-border" />
							</Tooltip.Content>
						</Tooltip.Portal>
					</Tooltip.Root>
				</Tooltip.Provider>
			{:else}
				<Tabs.Trigger value={id} class="data-[state=active]:bg-elevated p-1 rounded">
					{@render navLabel(section)}
				</Tabs.Trigger>
			{/if}
		{/each}
	</div>
	<div class={{ transition: true, 'opacity-100': mouseover, 'opacity-0': !mouseover }}>
		<Button.Root
			class="relative hover:bg-elevated p-1 rounded"
			aria-label="Toggle sidebar"
			onclick={() => (isSidebarOpen = !isSidebarOpen)}
		>
			<Icon
				icon="material-symbols:left-panel-close-outline-rounded"
				class={{
					'size-6 absolute transition': true,
					'opacity-100': isSidebarOpen,
					'opacity-0': !isSidebarOpen,
				}}
			/>
			<Icon
				icon="material-symbols:left-panel-open-outline-rounded"
				class={{
					'size-6 transition': true,
					'opacity-100': !isSidebarOpen,
					'opacity-0': isSidebarOpen,
				}}
			/>
		</Button.Root>
	</div>
</Tabs.List>
