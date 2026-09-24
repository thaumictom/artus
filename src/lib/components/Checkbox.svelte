<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Checkbox, type WithoutChildrenOrChild } from 'bits-ui';
	import { cn } from '$lib/utils';

	let {
		checked = $bindable(false),
		ref = $bindable(null),
		class: className,
		...restProps
	}: WithoutChildrenOrChild<Checkbox.RootProps> = $props();
</script>

<Checkbox.Root
	{...restProps}
	bind:checked
	bind:ref
	class={cn(
		'group inline-flex size-5 shrink-0 items-center justify-center border border-border bg-background text-background transition-colors cursor-pointer hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:border-accent data-[state=checked]:bg-accent',
		className,
	)}
>
	<Icon icon="material-symbols:check-rounded" class="size-4 opacity-0 transition-opacity group-data-[state=checked]:opacity-100" />
</Checkbox.Root>
