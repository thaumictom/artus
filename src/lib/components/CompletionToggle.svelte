<script lang="ts">
	import Checkbox from './Checkbox.svelte';
	let {
		completed,
		disabled = false,
		label,
		onCompletedChange,
	}: {
		completed: boolean;
		disabled?: boolean;
		label: string;
		onCompletedChange: (completed: boolean) => void;
	} = $props();
</script>

<label
	class="inline-flex shrink-0 items-center gap-2 rounded-full border px-2.5 py-1 text-sm font-medium transition-colors {completed
		? 'border-accent/30 bg-accent/10 text-accent hover:bg-accent/15'
		: 'border-border-secondary bg-surface/30 text-muted-foreground hover:bg-surface hover:text-foreground'} {disabled
		? 'cursor-not-allowed'
		: 'cursor-pointer'}"
>
	<Checkbox
		checked={completed}
		{disabled}
		class="data-[state=checked]:bg-transparent data-[state=checked]:border-accent/40 rounded-sm size-4 data-[state=checked]:text-accent"
		aria-label={`Mark ${label} as ${completed ? 'incomplete' : 'completed'}`}
		onCheckedChange={onCompletedChange}
	/>
	<span class="sr-only sm:not-sr-only">Completed</span>
</label>
