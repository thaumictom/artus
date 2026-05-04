<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { config, updateSetting } from '$lib/settings.svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import Button from '$lib/components/Button.svelte';
	import Switch from '$lib/components/Switch.svelte';
	import Icon from '@iconify/svelte';

	let isSaving = $state(false);
	let status = $state<string | null>(null);
	let inputValue = $state(config.warframe_log_path);

	$effect(() => {
		inputValue = config.warframe_log_path;
	});

	function sanitizeInput(input: string): string {
		let sanitized = input.trim();
		while (
			sanitized.length >= 2 &&
			((sanitized.startsWith('"') && sanitized.endsWith('"')) ||
				(sanitized.startsWith("'") && sanitized.endsWith("'")))
		) {
			sanitized = sanitized.slice(1, -1).trim();
		}
		return sanitized;
	}

	async function save() {
		if (isSaving) return;

		const sanitizedPath = sanitizeInput(inputValue);
		inputValue = sanitizedPath;

		isSaving = true;
		status = null;

		try {
			const validPath = await invoke<string>('validate_warframe_log_path', { path: sanitizedPath });
			config.warframe_log_path = validPath;
			await updateSetting('warframe_log_path');
			inputValue = validPath;
			status = 'Saved';
		} catch (error) {
			status = String(error);
		} finally {
			isSaving = false;
		}
	}

	const openFileBrowser = async () => {
		const file = await open({
			multiple: false,
			directory: false,
			defaultPath: config.warframe_log_path,
			filters: [{ name: 'EE.log', extensions: ['log'] }],
		});

		if (file) {
			inputValue = file;
			save();
		}
	};

	const relicSetting = 'relic_reward_detection';
</script>

<CommonSetting
	title="Log location"
	description="Path to your Warframe EE.log file"
	align="vertical"
>
	<div class="border p-0.5 flex">
		<input
			type="text"
			bind:value={inputValue}
			class="p-1.5 flex-1"
			onchange={save}
			placeholder="%LocalAppData%\Warframe\EE.log"
			disabled={isSaving}
		/>
		<Button variant="surface" onclick={openFileBrowser} disabled={isSaving}>Browse</Button>
	</div>

	{#if status}
		<div class="text-xs text-muted-foreground text-center flex items-center justify-center gap-1">
			<Icon icon="material-symbols:info-outline-rounded" />
			<span>{status}</span>
		</div>
	{/if}
</CommonSetting>
<CommonSetting
	title="Automatic relic reward detection"
	description="Automatically shows the overlay when the relic reward screen appears. Requires EE.log path to be set."
	disabled={!Boolean(config.warframe_log_path) || (status !== 'Saved' && status !== null)}
>
	<Switch
		id="relic-reward-detection-toggle"
		onCheckedChange={() => updateSetting(relicSetting)}
		bind:checked={config[relicSetting]}
	/>
</CommonSetting>
