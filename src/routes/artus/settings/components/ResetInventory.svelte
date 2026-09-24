<script lang="ts">
	import AlertDialog from '$lib/components/AlertDialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import { resetInventory } from '$lib/inventory';

	let showConfirmation = $state(false);
	let resetting = $state(false);
	let error = $state('');

	async function confirmReset() {
		resetting = true;
		error = '';
		try {
			await resetInventory();
			showConfirmation = false;
		} catch (cause) {
			console.error('Could not reset inventory:', cause);
			error = 'Could not clear the inventory. Please try again.';
		} finally {
			resetting = false;
		}
	}
</script>

<CommonSetting
	title="Clear inventory"
	description="Remove every inventory item and its new item markers."
>
	<Button
		class="border-danger text-danger hover:bg-danger/10 shrink-0"
		disabled={resetting}
		onclick={() => (showConfirmation = true)}
	>
		Clear inventory
	</Button>
</CommonSetting>
{#if error}<p role="alert" class="text-danger text-sm">{error}</p>{/if}

<AlertDialog bind:open={showConfirmation}>
	{#snippet title()}Clear the entire inventory?{/snippet}
	{#snippet description()}
		This removes every item and quantity from your inventory, including new item markers.
	{/snippet}
	{#snippet dialogCancel()}<Button onclick={() => (showConfirmation = false)}>Cancel</Button>{/snippet}
	{#snippet dialogAction()}
		<Button class="border-danger bg-danger text-danger-foreground hover:bg-danger/80" disabled={resetting} onclick={confirmReset}>
			Clear inventory
		</Button>
	{/snippet}
</AlertDialog>
