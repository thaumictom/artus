<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import { marketAccount } from '$lib/market-account.svelte';
	import { fetchMarketListings } from '$lib/market-listings';

	let open = $state(false);
	let deleting = $state(false);
	let error = $state('');
	let result = $state('');

	async function deleteAll() {
		if (deleting || !marketAccount.session) return;
		deleting = true;
		error = '';
		result = '';
		let deleted = 0;
		try {
			const listings = await fetchMarketListings();
			for (const listing of listings) {
				await invoke('market_delete_listing', { id: listing.id });
				deleted++;
			}
			open = false;
			result = listings.length === 0 ? 'No listings to delete.' : `Deleted ${deleted} listings.`;
		} catch (cause) {
			error = `Deleted ${deleted} listings before the operation stopped: ${String(cause)}`;
		} finally {
			deleting = false;
		}
	}
</script>

<CommonSetting
		title="Delete all listings"
		description={marketAccount.session
			? 'Delete every buy and sell listing from your warframe.market account.'
			: 'Log in to warframe.market to delete your listings.'}
		disabled={!marketAccount.session}
	>
		<Button
			class="border-danger text-danger hover:bg-danger/10 shrink-0"
			disabled={deleting || !marketAccount.session}
			onclick={() => { error = ''; result = ''; open = true; }}
		>
			Delete all listings
		</Button>
	</CommonSetting>
	{#if result}<p role="status" class="text-muted-foreground text-sm">{result}</p>{/if}
	{#snippet title()}Delete all listings?{/snippet}
	{#snippet description()}
		This permanently deletes every buy and sell listing from your warframe.market account.
	{/snippet}
	{#snippet dialogClose()}<Button disabled={deleting}>Cancel</Button>{/snippet}
	{#snippet dialogActions()}
		<Button
			class="border-danger bg-danger text-danger-foreground hover:bg-danger/80"
			disabled={deleting || !marketAccount.session}
			onclick={deleteAll}
		>
			{deleting ? 'Deleting...' : 'Delete all listings'}
		</Button>
	{/snippet}
	<Dialog
		bind:open={() => open, (value) => { if (!deleting) open = value; }}
		{title}
		{description}
		{dialogClose}
		{dialogActions}
		contentProps={{ class: 'h-auto' }}
	>
		{#if error}<p role="alert" class="px-6 text-danger text-sm">{error}</p>{/if}
	</Dialog>
