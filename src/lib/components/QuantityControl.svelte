<script lang="ts">
	import Icon from '@iconify/svelte';

	let {
		value,
		label,
		onChange,
		step = 1,
		decrementDisabled = false,
		incrementDisabled = false,
		disabled = false,
	}: {
		value: number;
		label: string;
		onChange: (delta: number) => void;
		step?: number;
		decrementDisabled?: boolean;
		incrementDisabled?: boolean;
		disabled?: boolean;
	} = $props();
</script>

<div class="inline-flex items-center gap-2">
	<button
		type="button"
		class="flex items-center justify-center border border-border-secondary size-6 hover:bg-surface disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
		aria-label={`Decrease ${label} by ${step}`}
		disabled={disabled || decrementDisabled}
		onclick={() => onChange(-step)}
	><Icon icon="lucide:minus" class="size-3.5" /></button>
	<span class="min-w-6 text-center tabular-nums">{value}</span>
	<button
		type="button"
		class="flex items-center justify-center border border-border-secondary size-6 hover:bg-surface disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
		aria-label={`Increase ${label} by ${step}`}
		disabled={disabled || incrementDisabled}
		onclick={() => onChange(step)}
	><Icon icon="lucide:plus" class="size-3.5" /></button>
</div>
