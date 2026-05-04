<script lang="ts">
	import { Collapsible, type WithoutChild } from 'bits-ui';
	import type { Snippet } from 'svelte';

	type Props = WithoutChild<Collapsible.RootProps> & {
		button: Snippet<[open: boolean]>;
		content: Snippet<[open: boolean]>;
	};

	let {
		open = $bindable(false),
		ref = $bindable(null),
		button,
		content,
		...restProps
	}: Props = $props();
</script>

<Collapsible.Root bind:open bind:ref {...restProps}>
	<Collapsible.Trigger>{@render button?.(open)}</Collapsible.Trigger>
	<Collapsible.Content forceMount>{@render content?.(open)}</Collapsible.Content>
</Collapsible.Root>
