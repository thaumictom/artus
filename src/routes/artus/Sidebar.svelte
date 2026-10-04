<script lang="ts">
	import { Button, Tabs } from 'bits-ui';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import TooltipProvider from '$lib/components/TooltipProvider.svelte';
	import Icon from '@iconify/svelte';
	import type { Sections } from '$lib/types';
	import { marketAccount } from '$lib/market-account.svelte';
	import { gsap } from 'gsap';
	import { CustomEase } from 'gsap/CustomEase';

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
	gsap.registerPlugin(CustomEase);
	// This curve has a vertical tangent at its midpoint.
	const pillEase = CustomEase.create('sidebarPill', '1,0,0,1');

	function animateIndicator(node: HTMLDivElement) {
		const pill = node.querySelector<HTMLElement>('[data-active-indicator]')!;
		const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
		let targetY: number | null = null;
		let animation: gsap.core.Timeline | undefined;
		let playback: gsap.core.Tween | undefined;
		let frame = 0;

		function positionIndicator() {
			const marker = node.querySelector<HTMLElement>(
				'[data-state="active"] [data-sidebar-indicator]',
			);
			if (!marker) {
				playback?.kill();
				animation?.kill();
				gsap.set(pill, { opacity: 0 });
				targetY = null;
				return;
			}

			const y = marker.getBoundingClientRect().top - node.getBoundingClientRect().top;
			if (y === targetY && !reducedMotion.matches) return;
			playback?.kill();
			animation?.kill();
			if (targetY === null || reducedMotion.matches) {
				gsap.set(pill, { y, height: 16, opacity: 1 });
			} else {
				// Stretch during travel, then settle into the selected tab's short pill.
				const currentY = Number(gsap.getProperty(pill, 'y'));
				const stretch = Math.min(Math.abs(y - currentY) * 0.3, 16);
				animation = gsap
					.timeline({ paused: true, defaults: { ease: 'none' } })
					.to(pill, {
						y: (currentY + y) / 2 - stretch / 2,
						height: 16 + stretch,
						duration: 0.14,
					})
					.to(pill, { y, height: 16, duration: 0.24 });
				// Ease the playhead once across both phases to avoid a midpoint pause.
				playback = animation.tweenTo(animation.duration(), { ease: pillEase });
			}
			targetY = y;
		}

		function schedulePosition() {
			cancelAnimationFrame(frame);
			frame = requestAnimationFrame(positionIndicator);
		}

		// Follow the tab state for clicks, keyboard navigation, and external navigation.
		const observer = new MutationObserver(schedulePosition);
		observer.observe(node, {
			subtree: true,
			childList: true,
			attributes: true,
			attributeFilter: ['data-state'],
		});
		const resizeObserver = new ResizeObserver(schedulePosition);
		resizeObserver.observe(node);
		reducedMotion.addEventListener('change', schedulePosition);
		schedulePosition();

		return {
			destroy() {
				cancelAnimationFrame(frame);
				observer.disconnect();
				resizeObserver.disconnect();
				reducedMotion.removeEventListener('change', schedulePosition);
				playback?.kill();
				animation?.kill();
			},
		};
	}
</script>

{#snippet navLabel(section: Sections[string])}
	<div class="px-2.5 w-full h-full group-data-[state=active]:text-accent cursor-pointer">
		<div
			class="flex items-center gap-2.5 hover:bg-elevated/50 group-data-[state=active]:bg-accent/10! py-2.5 rounded-md"
		>
			<span
				data-sidebar-indicator
				aria-hidden="true"
				class="group-data-[state=active]:bg-transparent -ml-px rounded-full w-0.5 h-4 shrink-0"
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
	<div class="flex flex-co min-h-0 overflow-x-hidden overflow-y-auto">
		<div class="relative flex flex-col gap-y-0.5 w-full shrink-0" use:animateIndicator>
			<span
				data-active-indicator
				aria-hidden="true"
				class="top-0 left-[9px] z-10 absolute bg-accent opacity-0 rounded-full w-0.5 h-4 pointer-events-none"
			></span>
			<TooltipProvider delayDuration={600} skipDelayDuration={200}>
				{#each Object.entries(sections) as [id, section]}
					{@const unavailable = id === 'listings' && !marketAccount.session}
					<Tooltip side="right" disabled={isSidebarOpen && !unavailable}>
						{#snippet trigger({ props })}
							{#if unavailable}
								<span
									{...props}
									class={`${props.class ?? ''} block w-full cursor-not-allowed`}
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
							{:else}
								<Tabs.Trigger
									{...props}
									value={id}
									aria-label={section.label}
									class={`${props.class ?? ''} ${navItemClass}`}
								>
									{@render navLabel(section)}
								</Tabs.Trigger>
							{/if}
						{/snippet}
						{#snippet content()}
							{section.label}{#if unavailable}: Log in to warframe.market first to view your listings.{/if}
						{/snippet}
					</Tooltip>
				{/each}
			</TooltipProvider>
		</div>
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
