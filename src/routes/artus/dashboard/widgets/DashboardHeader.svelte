<script lang="ts">
	import Button from '$lib/components/Button.svelte';
	import { timeAgo } from '$lib/date';

	let {
		loading,
		error,
		worldTimestamp,
		fetchedAt,
		now,
		onReload,
	}: {
		loading: boolean;
		error: string | null;
		worldTimestamp?: Date;
		fetchedAt: number | null;
		now: number;
		onReload: () => void | Promise<void>;
	} = $props();
</script>

<div class="flex items-center gap-3">
	<Button onclick={onReload} disabled={loading}>
		{loading ? 'Loading...' : 'Reload'}
	</Button>
	{#if worldTimestamp}
		<time class="text-muted-foreground text-sm" datetime={worldTimestamp.toISOString()}>
			{worldTimestamp.toLocaleString()}
		</time>
	{/if}
	{#if fetchedAt !== null}
		<span class="text-muted-foreground text-sm">fetched {timeAgo(fetchedAt, now)}</span>
	{/if}
</div>

{#if error}<p class="text-danger text-sm">{error}</p>{/if}
