<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Switch, type WithoutChildrenOrChild } from 'bits-ui';

	let {
		checked = $bindable(false),
		ref = $bindable(null),
		...restProps
	}: WithoutChildrenOrChild<Switch.RootProps> = $props();
</script>

<Switch.Root
	{...restProps}
	bind:checked
	bind:ref
	class="inline-flex justify-center items-center data-[state=checked]:bg-surface disabled:opacity-50 border w-10 h-6 transition-colors cursor-pointer disabled:cursor-not-allowed shrink-0"
>
	<Switch.Thumb
		class="group block bg-elevated data-[state=checked]:bg-foreground size-4 transition data-[state=checked]:translate-x-2 data-[state=unchecked]:-translate-x-2 pointer-events-none shrink-0"
	>
		<Icon
			icon="material-symbols:check"
			class="absolute group-data-[state=unchecked]:opacity-0 size-4 text-elevated transition-opacity"
		/>
		<Icon
			icon="material-symbols:check-indeterminate-small-rounded"
			class="top-1/2 left-1/2 absolute group-data-[state=checked]:opacity-0 size-4 text-black transition-opacity -translate-1/2"
		/>
	</Switch.Thumb>
</Switch.Root>
