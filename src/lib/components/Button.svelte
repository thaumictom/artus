<script lang="ts">
	import { Button } from 'bits-ui';
	import type { ComponentProps, Snippet } from 'svelte';

	let { props, children }: { props: ComponentProps<typeof Button.Root>; children?: Snippet } =
		$props();
</script>

<Button.Root {...props}>
	{@render children?.()}
</Button.Root>
