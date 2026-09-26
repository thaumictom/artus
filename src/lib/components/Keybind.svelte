<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	let {
		value,
		children,
		class: className,
		...rest
	}: HTMLAttributes<HTMLElement> & {
		value?: string;
		children?: Snippet;
	} = $props();

	// <kbd class="inline-flex items-center bg-zinc-100 shadow-sm px-3 py-1.5 border border-zinc-400 border-b-4 active:border-b-2 rounded-lg font-sans font-medium text-zinc-900 text-sm transition-[transform,border-width] active:translate-y-[2px] cursor-pointer select-none">⌘ K</kbd>

	const keyClass =
		'inline-flex min-w-5 h-5 items-center justify-center border border-border-secondary border-b-2 rounded-sm bg-surface/90 px-1.5 text-[10px] font-semibold text-foreground tracking-wide font-sans';

	function label(key: string): string {
		const name = key.trim();
		if (/^Key[A-Z]$/i.test(name)) return name.slice(3).toUpperCase();
		if (/^Digit[0-9]$/i.test(name)) return name.slice(5);
		const aliases: Record<string, string> = {
			control: 'Ctrl',
			ctrl: 'Ctrl',
			alt: 'Alt',
			shift: 'Shift',
			meta: 'Super',
			super: 'Super',
			escape: 'Esc',
			esc: 'Esc',
			arrowup: 'Up',
			arrowdown: 'Down',
			arrowleft: 'Left',
			arrowright: 'Right',
		};
		return aliases[name.toLowerCase()] ?? name.toUpperCase();
	}
</script>

{#if children}
	<kbd class={cn(keyClass, className)} {...rest}>
		{@render children()}
	</kbd>
{:else if value}
	<span class={cn('inline-flex items-center gap-1', className)} {...rest}>
		{#each value.split('+').filter((part) => part.trim()) as key, index}
			{#if index > 0}<span aria-hidden="true">+</span>{/if}
			<kbd class={keyClass}>{label(key)}</kbd>
		{/each}
	</span>
{/if}
