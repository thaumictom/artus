<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import Button from '$lib/components/Button.svelte';
	import { reloadOracleBounties } from '$lib/oracle-bounties.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import ResetMasteryItems from '../components/ResetMasteryItems.svelte';
	import ResetInventory from '../components/ResetInventory.svelte';
	import DeleteAllListings from '../components/DeleteAllListings.svelte';
	let refreshing = $state(false);
	let coolingDown = $state(false);
	let refreshStatus = $state<string | null>(null);
	let lastFetchedAt = $state<number | null>(null);
	let description = $derived(
		`Reload item data, prices, wiki offerings, bounties, Arbitration and invasion missions now. Last fetched: ${lastFetchedAt === null ? 'Never' : new Date(lastFetchedAt).toLocaleString()}.`,
	);
	let cooldownTimer: ReturnType<typeof setTimeout> | undefined;
	onMount(() => {
		let disposed = false;
		let unlisten: (() => void) | undefined;
		void listen<number>('api_catalogs_fetched', (event) => {
			if (!disposed) lastFetchedAt = event.payload;
		}).then(async (cleanup) => {
			if (disposed) {
				cleanup();
				return;
			}
			unlisten = cleanup;
			try {
				const fetchedAt = await invoke<number | null>('get_api_catalogs_last_fetched');
				if (!disposed && fetchedAt !== null)
					lastFetchedAt = Math.max(lastFetchedAt ?? 0, fetchedAt);
			} catch (error) {
				console.error('Could not load API catalog fetch time:', error);
			}
		});
		return () => {
			disposed = true;
			unlisten?.();
			clearTimeout(cooldownTimer);
		};
	});
	async function refreshApis() {
		if (refreshing || coolingDown) return;
		refreshing = true;
		coolingDown = true;
		refreshStatus = null;
		clearTimeout(cooldownTimer);
		cooldownTimer = setTimeout(() => (coolingDown = false), 3000);
		try {
			await invoke('refresh_api_catalogs');
		} catch (error) {
			refreshStatus = `Some catalogs, wiki offerings or Oracle missions could not be refreshed: ${String(error)}`;
		} finally {
			await reloadOracleBounties();
			refreshing = false;
		}
	}
</script>

<div class="flex flex-col">
	<div>
		<CommonSetting title="Refresh API catalogs" {description}>
			<Button onclick={refreshApis} disabled={refreshing || coolingDown} class="whitespace-nowrap">
				{refreshing ? 'Refreshing APIs…' : 'Refresh APIs now'}
			</Button>
		</CommonSetting>
		{#if refreshStatus}<p role="status" class="text-danger text-base">{refreshStatus}</p>{/if}
	</div>
	<ResetMasteryItems />
	<ResetInventory />
	<DeleteAllListings />
</div>
