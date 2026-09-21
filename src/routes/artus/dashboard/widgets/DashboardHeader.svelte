<script lang="ts">
	import Button from '$lib/components/Button.svelte';
	import { timeAgo } from '$lib/date';
	import Icon from '@iconify/svelte';

	let {
		loading,
		reloadCoolingDown,
		error,
		worldTimestamp,
		fetchedAt,
		now,
		onReload,
	}: {
		loading: boolean;
		reloadCoolingDown: boolean;
		error: string | null;
		worldTimestamp?: Date;
		fetchedAt: number | null;
		now: number;
		onReload: () => void | Promise<void>;
	} = $props();
</script>

<div class="flex justify-between items-center gap-3">
	<Button
		onclick={onReload}
		disabled={loading || reloadCoolingDown}
		class="flex items-center gap-1 text-sm"
	>
		<Icon icon="material-symbols:refresh" class={loading ? 'size-4 animate-spin' : 'size-4'} />
		{loading ? 'Refreshing...' : 'Reload dashboard'}
	</Button>
	<div class="text-right leading-0">
		{#if fetchedAt !== null}
			<div class="text-muted-foreground text-sm">fetched {timeAgo(fetchedAt, now)}</div>
		{/if}
		{#if worldTimestamp}
			<time class="text-muted-foreground text-xs" datetime={worldTimestamp.toISOString()}>
				snapshot: {worldTimestamp.toLocaleTimeString()}
			</time>
		{/if}
	</div>
</div>

{#if error}<p class="text-danger text-sm">{error}</p>{/if}
