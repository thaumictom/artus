<script lang="ts" generics="T extends string">
	import { ToggleGroup as BitsToggleGroup } from 'bits-ui';
	import { cn } from '$lib/utils';

	type ToggleOption<T extends string = string> = {
		value: T;
		label: string;
		disabled?: boolean;
	};

	let {
		options,
		value = $bindable(),
		label,
		class: className,
	}: {
		options: readonly ToggleOption<T>[];
		value: T[];
		label: string;
		class?: string;
	} = $props();
</script>

<BitsToggleGroup.Root
	type="multiple"
	aria-label={label}
	class={cn('inline-flex flex-wrap gap-0.5 p-0.5 border w-max', className)}
	bind:value
>
	{#each options as option (option.value)}
		<BitsToggleGroup.Item
			value={option.value}
			disabled={option.disabled}
			class="data-[state=on]:bg-surface disabled:opacity-50 px-2.5 py-1 text-muted-foreground data-[state=on]:text-surface-foreground text-sm cursor-pointer disabled:cursor-not-allowed"
		>
			{option.label}
		</BitsToggleGroup.Item>
	{/each}
</BitsToggleGroup.Root>
