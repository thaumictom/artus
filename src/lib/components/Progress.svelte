<script lang="ts">
	import { Progress, mergeProps, type WithoutChildrenOrChild } from 'bits-ui';
	import { cn } from '$lib/utils';

	let {
		value = 0,
		min = 0,
		max = 100,
		ref = $bindable(null),
		class: className,
		indicatorClass,
		indicatorColor,
		trackColor,
		gap = 0,
		...restProps
	}: WithoutChildrenOrChild<Progress.RootProps> & {
		indicatorClass?: string;
		indicatorColor?: string;
		trackColor?: string;
		gap?: number;
	} = $props();

	let normalizedValue = $derived(
		value !== null && Number.isFinite(value) ? Math.min(max, Math.max(min, value)) : null,
	);
	let percentage = $derived(
		normalizedValue !== null && max > min ? (normalizedValue - min) / (max - min) * 100 : 0,
	);
	let separated = $derived(gap > 0 && normalizedValue !== null);
	// At either endpoint the remaining segment disappears without leaving a gap.
	let halfGap = $derived(separated && percentage > 0 && percentage < 100 ? gap / 2 : 0);
</script>

<Progress.Root
	{...mergeProps(restProps, { style: { backgroundColor: separated ? 'transparent' : trackColor } })}
	bind:ref
	value={normalizedValue}
	{min}
	{max}
	class={cn('relative h-1.5 w-full overflow-hidden rounded-full bg-elevated', className)}
>
	{#if separated}
		<div
			aria-hidden="true"
			class="absolute inset-y-0 rounded-full bg-elevated transition-[left,width] duration-300 motion-reduce:transition-none"
			style:left={`calc(${percentage}% + ${halfGap}px)`}
			style:width={`max(0px, calc(${100 - percentage}% - ${halfGap}px))`}
			style:background-color={trackColor}
		></div>
	{/if}
	<div
		aria-hidden="true"
		class={cn(
			'h-full rounded-full bg-accent transition-[width] duration-300 motion-reduce:transition-none',
			normalizedValue === null && 'indeterminate',
			indicatorClass,
		)}
		style:width={normalizedValue === null ? '40%' : `max(0px, calc(${percentage}% - ${halfGap}px))`}
		style:background-color={indicatorColor}
	></div>
</Progress.Root>

<style>
	.indeterminate {
		animation: progress-indeterminate 1.5s ease-in-out infinite;
	}

	@keyframes progress-indeterminate {
		from { transform: translateX(-100%); }
		to { transform: translateX(250%); }
	}

	@media (prefers-reduced-motion: reduce) {
		.indeterminate { animation: none; }
	}
</style>
