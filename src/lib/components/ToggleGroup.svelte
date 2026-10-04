<script lang="ts" generics="T extends string">
	import { ToggleGroup as BitsToggleGroup } from 'bits-ui';
	import Icon from '@iconify/svelte';
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
	class={cn('inline-flex flex-wrap items-center gap-2 max-w-full', className)}
	bind:value
>
	{#each options as option (option.value)}
		<BitsToggleGroup.Item
			value={option.value}
			disabled={option.disabled}
			class="inline-flex items-center gap-1.5 rounded-full border border-border-secondary bg-surface/30 px-3 py-1.5 text-sm font-medium text-muted-foreground transition-colors enabled:hover:border-muted-foreground enabled:hover:bg-surface enabled:hover:text-foreground data-[state=on]:border-accent/60 data-[state=on]:bg-accent/15 data-[state=on]:text-accent enabled:data-[state=on]:hover:bg-accent/25 enabled:data-[state=on]:hover:text-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
		>
			<Icon
				icon={value.includes(option.value) ? 'lucide:check' : 'lucide:plus'}
				class="size-3.5 shrink-0"
				aria-hidden="true"
			/>
			{option.label}
		</BitsToggleGroup.Item>
	{/each}
</BitsToggleGroup.Root>
