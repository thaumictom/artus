<script lang="ts">
	import type { Snippet } from 'svelte';
	import { formatTimeLeft } from '$lib/date';

	let { children, action, footer, footerAside, completed = false, expiry, now, onActivate, disabled = false }: {
		children: Snippet;
		action?: Snippet;
		footer?: Snippet;
		footerAside?: Snippet;
		completed?: boolean;
		expiry?: Date;
		now: number;
		onActivate?: () => void;
		disabled?: boolean;
	} = $props();
	let hasExpiry = $derived(expiry instanceof Date && Number.isFinite(expiry.getTime()));

	function activate(event: MouseEvent) {
		if (!onActivate || disabled) return;
		// Links and controls own their clicks; the checkbox must not toggle twice.
		if (event.target instanceof Element && event.target.closest(
			'a, button, input, select, textarea, label, [role="button"], [role="checkbox"]',
		)) return;
		onActivate();
	}
</script>

<!-- Render inside a list; each view supplies its content and controls. -->
<!-- Card clicks supplement the keyboard-accessible control supplied in the action slot. -->
<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
<li onclick={activate} class:cursor-pointer={!!onActivate && !disabled}
	class="border transition-colors {completed ? 'bg-accent/5 border-accent/25' : 'bg-background border-border-secondary'}">
	<div class="flex items-start justify-between gap-4 p-4">
		<div class="min-w-0 flex-1">{@render children()}</div>
		{#if action}{@render action()}{/if}
	</div>
	{#if footer || footerAside || hasExpiry}
		<div class="flex flex-wrap items-center justify-between gap-x-4 gap-y-2 border-t border-surface bg-surface/20 px-4 py-3">
			{#if footer}<div class="min-w-0">{@render footer()}</div>{/if}
			{#if footerAside}{@render footerAside()}{/if}
			{#if hasExpiry && expiry}
				<span class="text-sm text-muted-foreground tabular-nums whitespace-nowrap">
					{#if expiry.getTime() > now}
						Ends in <time datetime={expiry.toISOString()} title={expiry.toLocaleString()}>{formatTimeLeft(expiry, now)}</time>
					{:else}Schedule updating{/if}
				</span>
			{/if}
		</div>
	{/if}
</li>
