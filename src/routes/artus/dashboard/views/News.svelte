<script lang="ts">
	import { openUrl } from '@tauri-apps/plugin-opener';
	import type { DashboardViewProps } from './view-types';

	let { world }: DashboardViewProps = $props();
	let articles = $derived(world.news);
	let linkError = $state(false);

	async function openArticle(url: string) {
		try {
			await openUrl(url);
			linkError = false;
		} catch {
			linkError = true;
		}
	}
</script>

<section class="max-w-3xl">
	<h2 class="mb-3 font-medium text-lg">News</h2>
	{#if linkError}<p class="mb-3 text-danger text-sm">Could not open the news article.</p>{/if}
	<ul class="bg-background border border-surface divide-y divide-surface">
		{#each articles as article}
			<li class="flex justify-between items-start gap-4 p-4">
				<div class="min-w-0">
					<button
						type="button"
						class="hover:text-accent disabled:hover:text-inherit text-left disabled:cursor-default"
						disabled={!article.link}
						onclick={() => openArticle(article.link)}
					>
						{article.message}
					</button>
					<div class="flex flex-wrap gap-2 mt-1 text-muted-foreground text-xs">
						<time datetime={article.date.toISOString()}>{article.date.toLocaleString()}</time>
						{#if article.priority}<span>Featured</span>{/if}
						{#if article.mobileOnly}<span>Mobile</span>{/if}
					</div>
				</div>
			</li>
		{:else}
			<li class="p-4 text-muted-foreground text-sm">No news articles in this snapshot.</li>
		{/each}
	</ul>
</section>
