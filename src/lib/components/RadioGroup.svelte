<script lang="ts" generics="T extends string">
	import { RadioGroup as BitsRadioGroup } from 'bits-ui';
	import { cn } from '$lib/utils';
	import type { Snippet } from 'svelte';

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
		disabled = false,
		onValueChange,
		optionContent,
		itemClass,
	}: {
		options: readonly RadioOption<T>[];
		value: T | '';
		label: string;
		variant?: 'segmented' | 'tabs' | 'cards' | 'swatches';
		separatorBefore?: number;
		class?: string;
		disabled?: boolean;
		onValueChange?: (value: T) => void;
		optionContent?: Snippet<[RadioOption<T>]>;
		itemClass?: string;
	} = $props();
</script>

<BitsRadioGroup.Root
	aria-label={label}
	class={cn(
		variant === 'segmented' ? 'flex p-0.5 border w-max' : 'flex flex-wrap gap-2',
		className,
	)}
	bind:value
	{disabled}
	onValueChange={(next) => {
		const option = options.find(({ value }) => value === next);
		if (option) onValueChange?.(option.value);
	}}
>
	{#if variant === 'segmented'}
		<div
			class="inline-flex flex-wrap gap-0.5 *:data-[state=checked]:bg-surface *:px-2.5 *:py-1 text-muted-foreground *:data-[state=checked]:text-surface-foreground *:text-sm *:cursor-pointer"
		>
			{#each options as option (option.value)}
				<BitsRadioGroup.Item value={option.value} disabled={option.disabled} class={itemClass}>
					{#if optionContent}{@render optionContent(option)}{:else}{option.label}{/if}
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
				aria-label={option.label}
				title={option.label}
				class={cn(
					'disabled:opacity-50 border focus-visible:outline-2 focus-visible:outline-ring focus-visible:outline-offset-2 cursor-pointer disabled:cursor-not-allowed',
					variant === 'swatches'
						? 'flex size-7 items-center justify-center rounded-full border-transparent data-[state=checked]:border-foreground'
						: 'data-[state=checked]:bg-accent/10 hover:bg-surface px-2 py-1.5 data-[state=checked]:border-accent text-muted-foreground data-[state=checked]:text-accent hover:text-foreground text-base',
					variant === 'cards' && 'flex min-w-0 flex-col items-center gap-2 p-2.5 text-sm',
					itemClass,
				)}
			>
				{#if optionContent}{@render optionContent(option)}{:else}{option.label}{/if}
			</BitsRadioGroup.Item>
		{/each}
	{/if}
</BitsRadioGroup.Root>
