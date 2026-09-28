<script lang="ts">
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	type Props = HTMLAttributes<HTMLDivElement> & { ref?: HTMLDivElement | null };
	let { ref = $bindable(null), class: className, ...restProps }: Props = $props();
</script>

<div
	bind:this={ref}
	data-slot="skeleton"
	aria-hidden="true"
	class={cn('bg-elevated rounded-md animate-pulse motion-reduce:animate-none', className)}
	{...restProps}
></div>
