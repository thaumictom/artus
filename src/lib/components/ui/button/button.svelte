<script lang="ts">
	import { cva, type VariantProps } from 'class-variance-authority';
	import { cn } from '$lib/utils';

	export const buttonVariants = cva(
		'inline-flex justify-center items-center disabled:opacity-50 rounded-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-black ring-offset-white focus-visible:ring-offset-2 font-medium text-sm whitespace-nowrap transition-colors disabled:pointer-events-none',
		{
			variants: {
				variant: {
					default: 'bg-black text-white hover:bg-black/90',
					secondary: 'bg-zinc-100 text-zinc-900 hover:bg-zinc-200',
					outline: 'border border-zinc-300 bg-white hover:bg-zinc-100',
					ghost: 'hover:bg-zinc-100',
				},
				size: {
					default: 'h-9 px-4 py-2',
					sm: 'h-8 rounded-md px-3',
					lg: 'h-10 rounded-md px-8',
				},
			},
			defaultVariants: {
				variant: 'default',
				size: 'default',
			},
		},
	);

	type Variant = VariantProps<typeof buttonVariants>['variant'];
	type Size = VariantProps<typeof buttonVariants>['size'];

	export let variant: Variant = 'default';
	export let size: Size = 'default';
	export let type: 'button' | 'submit' | 'reset' = 'button';
	export let className = '';
</script>

<button {type} class={cn(buttonVariants({ variant, size }), className)} {...$$restProps}>
	<slot />
</button>
