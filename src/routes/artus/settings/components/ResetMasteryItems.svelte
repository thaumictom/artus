<script lang="ts">
	import AlertDialog from '$lib/components/AlertDialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import { mastery, resetMasteryItems } from '$lib/mastery.svelte';

	let showConfirmation = $state(false);

	function confirmReset() {
		resetMasteryItems();
		showConfirmation = false;
	}
</script>

<CommonSetting
	title="Reset all mastery items"
	description="Clear every checked mastery item and its automatic marker. Other mastery XP is kept."
>
	<Button
		class="border-danger text-danger hover:bg-danger/10 shrink-0"
		disabled={mastery.loading}
		onclick={() => (showConfirmation = true)}
	>
		Reset items
	</Button>
</CommonSetting>

<AlertDialog bind:open={showConfirmation}>
	{#snippet title()}Reset all mastery items?{/snippet}
	{#snippet description()}
		This clears the checked state for every item and component. Your manually entered other mastery XP is kept.
	{/snippet}
	{#snippet dialogCancel()}<Button onclick={() => (showConfirmation = false)}>Cancel</Button>{/snippet}
	{#snippet dialogAction()}
		<Button class="border-danger bg-danger text-danger-foreground hover:bg-danger/80" onclick={confirmReset}>
			Reset items
		</Button>
	{/snippet}
</AlertDialog>
