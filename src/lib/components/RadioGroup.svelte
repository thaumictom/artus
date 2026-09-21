<script lang="ts" generics="T extends string">
	import { RadioGroup as BitsRadioGroup } from 'bits-ui';
	import { cn } from '$lib/utils';

	type RadioOption<T extends string = string> = {
		value: T;
		label: string;
		disabled?: boolean;
	};

	let {
		options,
		value = $bindable(),
		label,
		variant = 'segmented',
		separatorBefore,
		class: className,
	}: {
		options: readonly RadioOption<T>[];
		value: T;
		label: string;
		variant?: 'segmented' | 'tabs';
		separatorBefore?: number;
		class?: string;
	} = $props();
</script>

<BitsRadioGroup.Root
	aria-label={label}
	class={cn(
		variant === 'segmented' ? 'flex p-0.5 border w-max' : 'flex flex-wrap gap-2',
		className,
	)}
	bind:value
>
	{#if variant === 'segmented'}
		<div
			class="inline-flex flex-wrap gap-0.5 *:data-[state=checked]:bg-surface *:px-2.5 *:py-1 *:text-xs *:cursor-pointer"
		>
			{#each options as option (option.value)}
				<BitsRadioGroup.Item value={option.value} disabled={option.disabled}>
					{option.label}
				</BitsRadioGroup.Item>
			{/each}
		</div>
	{:else}
		{#each options as option, index (option.value)}
			{#if separatorBefore !== undefined && index === separatorBefore}
				<div aria-hidden="true" class="self-stretch bg-surface w-px min-h-7"></div>
			{/if}
			<BitsRadioGroup.Item
				value={option.value}
				disabled={option.disabled}
				class="data-[state=checked]:bg-accent/10 hover:bg-surface px-2 py-1.5 border data-[state=checked]:border-accent text-muted-foreground data-[state=checked]:text-accent hover:text-foreground text-sm cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
			>
				{option.label}
			</BitsRadioGroup.Item>
		{/each}
	{/if}
</BitsRadioGroup.Root>
