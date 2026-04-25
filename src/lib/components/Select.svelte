<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Select, type WithoutChildren } from 'bits-ui';

	type Props = WithoutChildren<Select.RootProps> & {
		placeholder?: string;
		items: { value: string; label: string; disabled?: boolean }[];
		contentProps?: WithoutChildren<Select.ContentProps>;
		// any other specific component props if needed
	};

	let { value = $bindable(), items, contentProps, placeholder, ...restProps }: Props = $props();
</script>

<!--
TypeScript Discriminated Unions + destructing (required for "bindable") do not
get along, so we shut typescript up by casting `value` to `never`, however,
from the perspective of the consumer of this component, it will be typed appropriately.
-->
<Select.Root bind:value={value as never} {...restProps}>
	<Select.Trigger
		class="flex justify-between p-2 border w-full max-w-80 cursor-pointer"
		aria-label="Select a theme"
	>
		<Select.Value placeholder="Select a theme" />
		<Icon icon="material-symbols:unfold-more-rounded" class="size-5" />
	</Select.Trigger>
	<Select.Portal>
		<Select.Content
			{...contentProps}
			class="border min-w-(--bits-select-anchor-width) backdrop-blur bg-surface/50 max-h-56"
			sideOffset={4}
		>
			<!-- <Select.ScrollUpButton>up</Select.ScrollUpButton> -->
			<Select.Viewport>
				{#each items as { value, label, disabled } (value)}
					<Select.Item
						{value}
						{label}
						{disabled}
						class="flex justify-between items-center hover:bg-elevated p-2 cursor-pointer"
					>
						{#snippet children({ selected })}
							<span>{label}</span>
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
