<script lang="ts">
	import { openUrl } from '@tauri-apps/plugin-opener';
	import Icon from '@iconify/svelte';
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import ViewCard from '$lib/components/ViewCard.svelte';
	import ViewToolbar from '$lib/components/ViewToolbar.svelte';
	import { timeAgo } from '$lib/date';
	import { validDate, type DashboardViewProps } from './view-types';

	let { world, now }: DashboardViewProps = $props();
	let articles = $derived(
		[...(world.news ?? [])].sort((a, b) => articleTimestamp(b.date) - articleTimestamp(a.date)),
	);
	let linkError = $state(false);
	let activeTab = $state('game');
	const newsTabs = [
		{ value: 'game', label: 'Game news' },
		{ value: 'community', label: 'Community' },
	] as const;
	let visibleArticles = $derived(
		articles.filter((article) => article.community === (activeTab === 'community')),
	);

	function articleTimestamp(date: Date): number {
		// Missing publication dates sort below dated articles.
		return validDate(date) && date.getTime() !== 0 ? date.getTime() : -Infinity;
	}

	async function openArticle(url: string) {
		try {
			await openUrl(url);
			linkError = false;
		} catch {
			linkError = true;
		}
	}
</script>

{#snippet articleList(entries: typeof articles)}
	<ul class="flex flex-col gap-3">
		{#each entries as article (article)}
			{@const showDate = validDate(article.date) && article.date.getTime() !== 0}
			{#snippet metadata()}
				<div class="flex flex-wrap items-center gap-3 text-muted-foreground text-sm">
					{#if showDate}
						<time datetime={article.date.toISOString()} title={article.date.toLocaleString()}>
							{timeAgo(article.date.getTime(), now)}
						</time>
					{/if}
					{#if article.priority}<span>Featured</span>{/if}
					{#if article.mobileOnly}<span>Mobile</span>{/if}
				</div>
			{/snippet}
			<ViewCard
				{now}
				disabled={!article.link}
				onActivate={() => {
					if (article.link) void openArticle(article.link);
				}}
				footer={showDate || article.priority || article.mobileOnly ? metadata : undefined}
			>
				<button
					type="button"
					class="flex justify-between items-start gap-4 focus-visible:outline-2 focus-visible:outline-accent focus-visible:outline-offset-2 w-full font-semibold text-left cursor-pointer disabled:cursor-default"
					disabled={!article.link}
					onclick={() => openArticle(article.link)}
				>
					<span class="min-w-0 break-words">{article.message}</span>
					{#if article.link}<Icon
							icon="lucide:external-link"
							class="mt-0.5 size-4 text-muted-foreground shrink-0"
						/>{/if}
				</button>
			</ViewCard>
		{:else}
			<li class="p-6 border border-surface text-muted-foreground text-sm text-center">
				No news articles in this snapshot.
			</li>
		{/each}
	</ul>
{/snippet}

<section class="flex flex-col gap-4 min-w-0" aria-label="News">
	<ViewToolbar stats={[{ value: visibleArticles.length, label: 'articles' }]}>
		{#snippet actions()}
			<RadioGroup
				label="News category"
				variant="segmented"
				options={newsTabs}
				bind:value={activeTab}
			/>
		{/snippet}
	</ViewToolbar>
	{#if linkError}<p role="alert" class="text-danger text-sm">
			Could not open the news article.
		</p>{/if}
	{@render articleList(visibleArticles)}
</section>
