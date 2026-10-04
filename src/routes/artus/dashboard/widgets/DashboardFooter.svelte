<script lang="ts">
	import type { Snippet } from 'svelte';
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
		children,
	}: {
		loading: boolean;
		reloadCoolingDown: boolean;
		error: string | null;
		worldTimestamp?: Date;
		fetchedAt: number | null;
		now: number;
		onReload: () => void | Promise<void>;
		children?: Snippet;
	} = $props();
</script>

<div class="flex flex-col gap-2 px-6 pb-4.5">
	{#if error}<p class="text-danger text-sm wrap-break-word">{error}</p>{/if}
	<div class="flex flex-col text-muted-foreground text-sm">
		{#if fetchedAt !== null}
			<div>fetched {timeAgo(fetchedAt, now)}</div>
		{/if}
		{#if worldTimestamp}
			<time datetime={worldTimestamp.toISOString()} class="text-xs">
				snapshot: {worldTimestamp.toLocaleTimeString()}
			</time>
		{/if}
	</div>
	<div class="flex items-stretch gap-1">
		<Button
			onclick={onReload}
			disabled={loading || reloadCoolingDown}
			class="flex flex-1 justify-center items-center gap-1 px-2 min-w-0 text-sm"
		>
			<Icon
				icon="material-symbols:refresh"
				class={loading ? 'size-4 shrink-0 animate-spin' : 'size-4 shrink-0'}
			/>
			{loading ? 'Refreshing...' : 'Reload state'}
		</Button>
		{@render children?.()}
	</div>
</div>
