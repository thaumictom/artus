<script lang="ts">
	let { colors }: { colors: readonly [string, string] } = $props();
</script>

<span
	class="inline-flex border border-border-secondary rounded-full w-8 h-3 overflow-hidden shrink-0"
	aria-hidden="true"
>
	<span class="w-1/2 h-full" style:background-color={colors[0]}></span>
	<span class="w-1/2 h-full" style:background-color={colors[1]}></span>
</span>
