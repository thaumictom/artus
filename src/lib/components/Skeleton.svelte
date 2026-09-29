<script lang="ts">
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLAttributes<HTMLDivElement> & {
		ref?: HTMLDivElement | null;
		children?: import('svelte').Snippet;
	};
	let { ref = $bindable(null), class: className, children, ...restProps }: Props = $props();
</script>

<div
	bind:this={ref}
	data-slot="skeleton"
	aria-hidden="true"
	class={cn('bg-surface text-trim animate-pulse motion-reduce:animate-none', className)}
	{...restProps}
>
	{#if children}
		<div class="invisible whitespace-nowrap">{@render children?.()}</div>
	{/if}
</div>
