<script lang="ts">
	import { Label, type WithoutChildrenOrChild } from 'bits-ui';
	import type { ComponentProps, Snippet } from 'svelte';

	type Props = {
		children?: Snippet;
		disabled?: boolean;
		title: string;
		description: string;
		align?: 'horizontal' | 'vertical';
		labelProps?: ComponentProps<WithoutChildrenOrChild<typeof Label.Root>>;
	};

	let {
		children,
		disabled,
		title,
		description,
		align = 'horizontal',
		labelProps,
	}: Props = $props();
</script>

<div
	class={{
		'cursor-not-allowed': disabled,
	}}
>
	<div
		class={{
			'flex transition-opacity text-left': true,
			'flex-col gap-2': align === 'vertical',
			'justify-between items-center gap-4': align === 'horizontal',
			'opacity-50 pointer-events-none': disabled,
		}}
	>
		<Label.Root {...labelProps}>
			<h2>{title}</h2>
			<p class="text-muted-foreground text-xs">
				{description}
			</p>
		</Label.Root>
		{@render children?.()}
	</div>
</div>
