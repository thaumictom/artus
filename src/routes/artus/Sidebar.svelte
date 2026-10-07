<script lang="ts">
	import { Tabs } from 'bits-ui';
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
	const DRAG_THRESHOLD = 24;
	let drag = $state<{
		pointerId: number;
		startX: number;
		anchorX: number;
		targetOpen: boolean;
		moved: boolean;
		triggered: boolean;
	} | null>(null);

	function startDrag(event: PointerEvent) {
		if (!canToggle || drag || event.button !== 0) return;
		const handle = event.currentTarget as HTMLButtonElement;
		event.preventDefault();
		drag = {
			pointerId: event.pointerId,
			startX: event.clientX,
			anchorX: event.clientX,
			targetOpen: isSidebarOpen,
			moved: false,
			triggered: false,
		};
		handle.setPointerCapture(event.pointerId);
	}

	function moveDrag(event: PointerEvent) {
		if (!drag || event.pointerId !== drag.pointerId) return;
		const distance = event.clientX - drag.startX;
		if (Math.abs(distance) >= 4) drag.moved = true;
		// Follow the furthest point so reversing 24px can interrupt the animation
		// immediately, even during the same held drag.
		drag.anchorX = drag.targetOpen
			? Math.max(drag.anchorX, event.clientX)
			: Math.min(drag.anchorX, event.clientX);
		const directionalDistance = drag.targetOpen
			? drag.anchorX - event.clientX
			: event.clientX - drag.anchorX;
		if (directionalDistance >= DRAG_THRESHOLD) {
			drag.triggered = true;
			drag.targetOpen = !drag.targetOpen;
			drag.anchorX = event.clientX;
			if (canToggle && isSidebarOpen !== drag.targetOpen) onToggle();
		}
	}

	function endDrag(event: PointerEvent) {
		if (!drag || event.pointerId !== drag.pointerId) return;
		moveDrag(event);
		const clicked = !drag.moved && !drag.triggered;
		cancelDrag();
		if (canToggle && clicked) onToggle();
	}

	function cancelDrag() {
		drag = null;
	}

	function resizeWithKeyboard(event: KeyboardEvent) {
		if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
		event.preventDefault();
		if (canToggle && (event.key === 'ArrowRight') !== isSidebarOpen) onToggle();
	}

	$effect(() => {
		if (!canToggle) cancelDrag();
	});

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
	<div class="px-2.5 w-full min-w-0 h-full group-data-[state=active]:text-accent cursor-pointer">
		<div
			class="flex items-center gap-2.5 hover:bg-elevated/50 group-data-[state=active]:bg-accent/10! py-2.5 rounded-md transition-colors duration-200"
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
					'overflow-hidden min-w-0 text-base whitespace-nowrap transition-opacity duration-200 motion-reduce:transition-none text-left flex-1': true,
					'opacity-100': isSidebarOpen,
					'opacity-0': !isSidebarOpen,
				}}
			>
				{section.label}
			</span>
		</div>
	</div>
{/snippet}

<div
	class="group/sidebar relative h-full min-h-0 sidebar shrink-0"
	data-dragging={drag ? '' : undefined}
	style:width={isSidebarOpen ? '12rem' : '4rem'}
>
	<Tabs.List
		aria-label="Main navigation"
		class="flex flex-col justify-between border-border-secondary w-full h-full min-h-0 overflow-hidden"
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
								{section.label}{#if unavailable}: Log in to warframe.market first to view your
									listings.{/if}
							{/snippet}
						</Tooltip>
					{/each}
				</TooltipProvider>
			</div>
		</div>
	</Tabs.List>
	{#if canToggle}
		<button
			type="button"
			aria-label={isSidebarOpen
				? 'Close sidebar: click or drag left'
				: 'Open sidebar: click or drag right'}
			aria-expanded={isSidebarOpen}
			class="-right-1 z-20 absolute inset-y-0 rounded focus-visible:outline-2 focus-visible:outline-accent w-2 touch-none cursor-ew-resize sidebar-resizer"
			onpointerdown={startDrag}
			onpointermove={moveDrag}
			onpointerup={endDrag}
			onpointercancel={cancelDrag}
			onlostpointercapture={cancelDrag}
			onkeydown={resizeWithKeyboard}
			onclick={(event) => {
				if (event.detail === 0) onToggle();
			}}
		>
			<span
				class="left-1/2 absolute inset-y-3 bg-accent opacity-0 rounded w-0.5 transition-opacity pointer-events-none"
			></span>
		</button>
	{/if}
</div>

<style>
	.sidebar {
		transition: width 220ms cubic-bezier(0.2, 0, 0, 1);
	}

	.sidebar-resizer:hover span,
	.sidebar-resizer:focus-visible span,
	.sidebar[data-dragging] .sidebar-resizer span {
		opacity: 0.6;
	}

	:global(body:has(.sidebar[data-dragging])),
	:global(body:has(.sidebar[data-dragging]) *) {
		cursor: ew-resize !important;
		user-select: none !important;
	}

	@media (prefers-reduced-motion: reduce) {
		.sidebar {
			transition: none;
		}
	}
</style>
