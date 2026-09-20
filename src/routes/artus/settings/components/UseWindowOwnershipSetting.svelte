<script lang="ts">
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import AlertDialog from '$lib/components/AlertDialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import { tick } from 'svelte';

	const setting = 'use_window_ownership';
	let showWarning = $state(false);
	let switchChecked = $state(config[setting]);

	$effect(() => {
		if (!showWarning) switchChecked = config[setting];
	});

	async function handleCheckedChange(checked: boolean) {
		if (checked) {
			showWarning = true;
			await tick();
			switchChecked = false;
			return;
		}

		config[setting] = false;
		void updateSetting(setting);
	}

	function cancelWindowOwnership() {
		switchChecked = false;
	}

	function applyWindowOwnership() {
		config[setting] = true;
		switchChecked = true;
		showWarning = false;
		void updateSetting(setting);
	}
</script>

<CommonSetting
	title="Use window ownership (experimental)"
	description="Attach the overlay directly to Warframe to prevent it from displaying over other apps when alt-tabbed, but can cause issues due to anti-cheat."
	align="horizontal"
	labelProps={{ for: setting }}
>
	<Switch id={setting} bind:checked={switchChecked} onCheckedChange={handleCheckedChange} />
</CommonSetting>

<AlertDialog bind:open={showWarning}>
	{#snippet title()}
		<div>Caution</div>
	{/snippet}
	{#snippet description()}
		<div>
			This setting modifies the game window, and Warframe can detect that a third-party app is
			attached to it. This behavior can be flagged by anti-cheat systems.
			<br />
			<br />
			Use at your own risk. Do you want to enable this setting?
		</div>
	{/snippet}
	{#snippet dialogCancel()}
		<Button onclick={cancelWindowOwnership} variant="default" tabindex={-1}>Cancel</Button>
	{/snippet}
	{#snippet dialogAction()}
		<Button onclick={applyWindowOwnership} variant="primary" tabindex={-1}>Enable</Button>
	{/snippet}
</AlertDialog>
