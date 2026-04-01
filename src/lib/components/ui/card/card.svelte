<script lang="ts">
	import { cn } from '$lib/utils';

	export let className = '';
</script>

<div class={cn('bg-white shadow-sm border rounded-lg text-zinc-950', className)} {...$$restProps}>
	<slot />
</div>
