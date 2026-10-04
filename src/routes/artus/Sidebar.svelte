<script lang="ts">
	import { Button, Tabs, Tooltip } from 'bits-ui';
	import Icon from '@iconify/svelte';
	import type { Sections } from '$lib/types';
	import { marketAccount } from '$lib/market-account.svelte';
	import { is } from 'zod/v4/locales';

	let {
		sections,
		isSidebarOpen,
		canToggle,
		onToggle,
	}: {
		sections: Sections;
		isSidebarOpen: boolean;
		canToggle: boolean;
		onToggle: () => void;
	} = $props();

	const navItemClass = 'group';
</script>

{#snippet navLabel(section: Sections[string])}
	<div class="px-2.5 w-full h-full group-data-[state=active]:text-accent cursor-pointer">
		<div class="flex items-center gap-2.5 group-data-[state=active]:bg-accent/10 py-2.5 rounded-md">
			<span
				class="group-data-[state=active]:bg-accent group-hover:bg-accent rounded-full w-0.5 h-4"
			></span>
			<Icon icon={section.icon} class="size-5 shrink-0" />
			<span
				aria-hidden={!isSidebarOpen}
				class={{
					'overflow-hidden text-base whitespace-nowrap transition-opacity text-left': true,
					'opacity-100 flex-1': isSidebarOpen,
					'opacity-0 w-0': !isSidebarOpen,
				}}
			>
				{section.label}
			</span>
		</div>
	</div>
{/snippet}

<Tabs.List
	aria-label="Main navigation"
	class={`group/sidebar flex flex-col justify-between bg-surface border-border-secondary h-full min-h-0 shrink-0 transition-[width] duration-300 ease-in-out ${isSidebarOpen ? 'w-48' : 'w-16'}`}
>
	<div class="flex flex-col min-h-0 overflow-x-hidden overflow-y-auto">
		{#each Object.entries(sections) as [id, section]}
			{#if id === 'listings' && !marketAccount.session}
				<Tooltip.Provider delayDuration={200}>
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<span
									{...props}
									class="block w-full cursor-not-allowed"
									aria-label="Listings unavailable. Log in to warframe.market first."
								>
									<Tabs.Trigger
										value={id}
										disabled
										aria-label={section.label}
										class={`${navItemClass} opacity-40 pointer-events-none`}
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
								class="z-100 bg-surface shadow-xl p-3 border border-border max-w-64 text-surface-foreground text-base"
							>
								Log in to warframe.market first to view your listings.
								<Tooltip.Arrow class="text-border" />
							</Tooltip.Content>
						</Tooltip.Portal>
					</Tooltip.Root>
				</Tooltip.Provider>
			{:else}
				<Tabs.Trigger
					value={id}
					aria-label={section.label}
					title={!isSidebarOpen ? section.label : undefined}
					class={navItemClass}
				>
					{@render navLabel(section)}
				</Tabs.Trigger>
			{/if}
		{/each}
	</div>
	<div class="max-[800px]:hidden p-3 shrink-0">
		<Button.Root
			class="relative hover:bg-elevated opacity-0 focus-visible:opacity-100 group-focus-within/sidebar:opacity-100 group-hover/sidebar:opacity-100 p-2 rounded focus-visible:outline-2 focus-visible:outline-accent text-muted-foreground hover:text-foreground transition cursor-pointer"
			aria-label="Toggle sidebar"
			aria-expanded={isSidebarOpen}
			disabled={!canToggle}
			onclick={onToggle}
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
