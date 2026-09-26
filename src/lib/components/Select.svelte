<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Select, type WithoutChildren } from 'bits-ui';
	import cn from 'clsx';

	type SelectItem = { value: string; label: string; disabled?: boolean; swatchColors?: [string, string] };
	type Props = WithoutChildren<Select.RootProps> & {
		placeholder?: string;
		items: SelectItem[];
		triggerProps?: WithoutChildren<Select.TriggerProps>;
		contentProps?: WithoutChildren<Select.ContentProps>;
		// any other specific component props if needed
	};

	let {
		value = $bindable(),
		items,
		contentProps,
		triggerProps,
		placeholder,
		...restProps
	}: Props = $props();
	const selectedItem = $derived((items as SelectItem[]).find((item) => item.value === value));
</script>

{#snippet colorPill(colors: [string, string])}
	<span class="inline-flex overflow-hidden rounded-full border border-border-secondary w-8 h-3 shrink-0" aria-hidden="true">
		<span class="w-1/2 h-full" style:background-color={colors[0]}></span>
		<span class="w-1/2 h-full" style:background-color={colors[1]}></span>
	</span>
{/snippet}

<!--
TypeScript Discriminated Unions + destructing (required for "bindable") do not
get along, so we shut typescript up by casting `value` to `never`, however,
from the perspective of the consumer of this component, it will be typed appropriately.
-->
<Select.Root bind:value={value as never} {items} {...restProps}>
	<Select.Trigger
		{...triggerProps}
		class={cn(
			'flex justify-between p-2 border w-full max-w-80 cursor-pointer',
			triggerProps?.class,
		)}
		aria-label={placeholder}
	>
		<span class="flex items-center gap-2 min-w-0">
			{#if selectedItem?.swatchColors}{@render colorPill(selectedItem.swatchColors)}{/if}
			<Select.Value {placeholder} />
		</span>
		<Icon icon="material-symbols:unfold-more-rounded" class="size-5" />
	</Select.Trigger>
	<Select.Portal>
		<Select.Content
			{...contentProps}
			class={cn(
				'z-50 border min-w-(--bits-select-anchor-width) backdrop-blur bg-surface/50 max-h-56',
				contentProps?.class,
			)}
			sideOffset={4}
		>
			<!-- <Select.ScrollUpButton>up</Select.ScrollUpButton> -->
			<Select.Viewport>
				{#each items as { value, label, disabled, swatchColors } (value)}
					<Select.Item
						{value}
						{label}
						{disabled}
						class="flex justify-between items-center hover:bg-elevated p-2 cursor-pointer"
					>
						{#snippet children({ selected })}
							<span class="flex items-center gap-2">
								{#if swatchColors}{@render colorPill(swatchColors)}{/if}
								{label}
							</span>
							{#if selected}
								<Icon icon="material-symbols:check" class="size-5" />
							{/if}
						{/snippet}
					</Select.Item>
				{/each}
			</Select.Viewport>
			<!-- <Select.ScrollDownButton>down</Select.ScrollDownButton> -->
		</Select.Content>
	</Select.Portal>
</Select.Root>
