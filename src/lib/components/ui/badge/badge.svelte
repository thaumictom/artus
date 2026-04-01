<script lang="ts">
	import { cva, type VariantProps } from 'class-variance-authority';
	import { cn } from '$lib/utils';

	const badgeVariants = cva(
		'inline-flex items-center px-2.5 py-0.5 border rounded-full font-semibold text-xs transition-colors',
		{
			variants: {
				variant: {
					default: 'border-transparent bg-black text-white',
					secondary: 'border-transparent bg-zinc-100 text-zinc-900',
					outline: 'border-zinc-300 text-zinc-900',
				},
			},
			defaultVariants: {
				variant: 'default',
			},
		},
	);

	type Variant = VariantProps<typeof badgeVariants>['variant'];

	export let variant: Variant = 'default';
	export let className = '';
</script>

<span class={cn(badgeVariants({ variant }), className)} {...$$restProps}>
	<slot />
</span>
