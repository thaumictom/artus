<script module lang="ts">
	const platinumFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });
	const integerFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });
</script>

<script lang="ts">
	import type { HTMLAttributes } from 'svelte/elements';
	import { cn } from '$lib/utils';

	let {
		value,
		currency,
		prefix = '',
		showValue = true,
		iconClass,
		class: className,
		...restProps
	}: Omit<HTMLAttributes<HTMLSpanElement>, 'children'> & {
		value: number | string | null | undefined;
		currency: 'credits' | 'platinum' | 'ducats';
		prefix?: string;
		showValue?: boolean;
		iconClass?: string;
	} = $props();

	// Missing or non-finite amounts stay unknown, while zero remains a real amount.
	const formattedValue = $derived(typeof value === 'number'
		? Number.isFinite(value)
			? (currency === 'platinum' ? platinumFormatter : integerFormatter).format(value)
			: '—'
		: value ?? '—');
</script>

<span class={cn('inline-flex items-center gap-1 whitespace-nowrap tabular-nums', className)} {...restProps}>
	{#if showValue}<span>{formattedValue === '—' ? '' : prefix}{formattedValue}</span>{/if}
	<img src={`/icons/${currency}.png`} alt={currency} class={cn('size-3.5 shrink-0 object-contain', iconClass)} />
</span>
