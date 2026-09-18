<script lang="ts">
	import Icon from '@iconify/svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { duration, type WorldSection } from '$lib/worldstate';

	let { section, now }: { section: WorldSection; now: number } = $props();
	let linkError = $state(false);
	async function openLink(url: string) {
		try { await openUrl(url); linkError = false; }
		catch { linkError = true; }
	}
</script>

<details id={section.id} open={!section.collapsed} class="group bg-background border border-surface scroll-mt-4">
	<summary class="flex items-center gap-2.5 hover:bg-surface/50 p-4 focus-visible:outline-2 focus-visible:outline-accent cursor-pointer list-none">
		<Icon icon={section.icon} class="size-5 text-accent shrink-0" />
		<h2 class="flex-1 font-medium">{section.title}</h2>
		<span class="text-muted-foreground text-xs tabular-nums">{section.rows.length}</span>
		<Icon icon="material-symbols:expand-more-rounded" class="size-5 text-muted-foreground group-open:rotate-180 transition-transform" />
	</summary>
	<div class="px-4 pb-4">
		{#if section.note}<p class="mb-3 text-muted-foreground text-xs">{section.note}</p>{/if}
		{#if linkError}<p role="alert" class="mb-2 text-danger text-sm">Could not open this link.</p>{/if}
		<ul class="divide-y divide-surface max-h-96 overflow-y-auto pr-1">
			{#each section.rows as row, index (index)}
				{@const expired = row.expiry !== undefined && row.expiry <= now}
				<li class="flex flex-col gap-1.5 py-3 first:pt-0 last:pb-0" class:opacity-60={expired}>
					<div class="flex justify-between items-start gap-2">
						{#if row.url}
							<button type="button" class="hover:text-accent focus-visible:outline-accent text-left text-sm cursor-pointer" onclick={() => openLink(row.url!)}>
								{row.title} <Icon icon="material-symbols:open-in-new-rounded" class="inline size-3" />
							</button>
						{:else}<h3 class="text-sm leading-snug">{row.title}</h3>{/if}
						{#if row.badge}<span class="bg-surface px-1.5 py-0.5 text-muted-foreground text-xs text-right shrink-0 max-w-36">{row.badge}</span>{/if}
					</div>
					{#if row.detail}<p class="text-muted-foreground text-xs leading-relaxed">{row.detail}</p>{/if}
					{#if row.rewards?.length}
						<ul class="flex flex-col gap-1 text-xs leading-relaxed">
							{#each row.rewards as reward}<li class="text-accent/90">{reward}</li>{/each}
						</ul>
					{/if}
					{#if row.progress !== undefined}
						<div class="flex justify-between gap-2 text-muted-foreground text-xs">
							<span>{row.progressLabel}</span><span>{row.progress.toFixed(1)}%</span>
						</div>
						<progress aria-label={row.progressLabel ?? 'Progress'} value={row.progress} max="100" class="w-full h-1 accent-accent"></progress>
					{/if}
					{#if row.activation !== undefined && row.activation > now}
						<p class="text-accent text-xs tabular-nums" title={new Date(row.activation).toLocaleString()}>Starts in {duration(row.activation - now)}</p>
					{:else if row.expiry !== undefined}
						<p class="text-muted-foreground text-xs tabular-nums" title={new Date(row.expiry).toLocaleString()}>
							{expired ? 'Ended · reload for current data' : `Ends in ${duration(row.expiry - now)}`}
						</p>
					{/if}
				</li>
			{:else}<li class="text-muted-foreground text-sm">{section.empty ?? 'No information in this snapshot.'}</li>{/each}
		</ul>
	</div>
</details>
