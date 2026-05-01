<script lang="ts">
	import type { ComponentProps, Snippet } from 'svelte';
	import { Slider } from 'bits-ui';
	import cn from 'clsx';

	let {
		value = $bindable(),
		ref = $bindable(null),
		children,
		class: className,
		thumbLabel,
		...restProps
	}: ComponentProps<typeof Slider.Root> & {
		thumbLabel?: Snippet<[{ value: number | number[] | undefined }]>;
	} = $props();
</script>

<div class="group flex items-center h-6">
	<Slider.Root
		bind:value
		bind:ref
		{...restProps as any}
		thumbPositioning="equal"
		class={cn(
			'relative flex items-center bg-surface border w-full h-1.5 has-data-active:h-2.5 group-hover:h-2.5 transition-[height] touch-none select-none',
			className,
		)}
		trackPadding={1.5}
	>
		{#snippet children({ thumbItems })}
			<Slider.Range class="absolute bg-foreground h-full" />
			{#each thumbItems as { index, value } (index)}
				<Slider.Thumb {index} class="group">
					<div
						class="bg-foreground border size-4.5 group-data-active:size-5.5 transition-all cursor-e-resize"
					></div>
				</Slider.Thumb>
				{#if thumbLabel}
					<Slider.ThumbLabel
						{index}
						position="bottom"
						class="bg-surface opacity-0 data-active:opacity-100 mt-3 px-2 border text-sm text-nowrap transition-opacity pointer-events-none"
					>
						{@render thumbLabel({ value })}
					</Slider.ThumbLabel>
				{/if}
			{/each}
		{/snippet}
	</Slider.Root>
</div>
