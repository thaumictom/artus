<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Checkbox, type WithoutChildrenOrChild } from 'bits-ui';
	import { cn } from '$lib/utils';
	type Props = WithoutChildrenOrChild<Checkbox.RootProps> & { owned?: boolean };

	let {
		checked = $bindable(false),
		owned = false,
		ref = $bindable(null),
		class: className,
		...restProps
	}: Props = $props();
</script>

<Checkbox.Root
	{...restProps}
	bind:checked
	bind:ref
	class={cn(
		'group relative inline-flex size-5 shrink-0 items-center justify-center border border-border bg-background text-background transition-colors cursor-pointer hover:border-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:border-accent data-[state=checked]:bg-accent data-[state=indeterminate]:border-muted-foreground data-[state=indeterminate]:text-muted-foreground',
		owned && !checked && 'border-muted-foreground bg-surface text-muted-foreground',
		className,
	)}
>
	<Icon icon="material-symbols:check-rounded" class={cn('absolute size-4 opacity-0 transition-opacity group-data-[state=checked]:opacity-100', owned && !checked && 'opacity-100')} />
	<Icon icon="material-symbols:remove-rounded" class="absolute size-4 opacity-0 transition-opacity group-data-[state=indeterminate]:opacity-100" />
</Checkbox.Root>
