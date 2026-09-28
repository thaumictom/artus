<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import Button from '$lib/components/Button.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import ResetMasteryItems from '../components/ResetMasteryItems.svelte';
	import ResetInventory from '../components/ResetInventory.svelte';
	import DeleteAllListings from '../components/DeleteAllListings.svelte';
	let refreshing = $state(false);
	let coolingDown = $state(false);
	let refreshStatus = $state<string | null>(null);
	let cooldownTimer: ReturnType<typeof setTimeout> | undefined;
	onMount(() => () => clearTimeout(cooldownTimer));
	async function refreshApis() {
		if (refreshing || coolingDown) return;
		refreshing = true;
		coolingDown = true;
		refreshStatus = null;
		clearTimeout(cooldownTimer);
		cooldownTimer = setTimeout(() => coolingDown = false, 3000);
		try {
			await invoke('refresh_api_catalogs');
			refreshStatus = 'API catalogs refreshed.';
		} catch (error) {
			refreshStatus = `Some API catalogs could not be refreshed: ${String(error)}`;
		} finally {
			refreshing = false;
		}
	}
</script>

<div class="flex flex-col gap-8">
	<div>
		<CommonSetting title="Refresh API catalogs" description="Reload item data and prices now. Artus also refreshes them every 30 minutes.">
		<Button onclick={refreshApis} disabled={refreshing || coolingDown}>
			{refreshing ? 'Refreshing APIs…' : 'Refresh APIs now'}
		</Button>
		</CommonSetting>
		{#if refreshStatus}<p role="status" class="text-sm text-muted-foreground">{refreshStatus}</p>{/if}
	</div>
	<ResetMasteryItems />
	<ResetInventory />
	<DeleteAllListings />
</div>
