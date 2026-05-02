<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { config, updateSetting } from '$lib/settings.svelte';
	import { open } from '@tauri-apps/plugin-dialog';

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
		});
	};
</script>

<div class="flex flex-col gap-2">
	<div>
		<p>Warframe log path</p>
		<p class="text-muted-foreground text-xs">Set this to your Warframe EE.log file path</p>
	</div>

	<div class="border p-0.5 flex">
		<input
			type="text"
			bind:value={inputValue}
			class="p-1.5 flex-1"
			onchange={save}
			placeholder="%LocalAppData%\Warframe\EE.log"
			disabled={isSaving}
		/>
		<button
			class="bg-surface group-hover:bg-muted px-3 transition"
			onclick={openFileBrowser}
			disabled={isSaving}
		>
			Browse
		</button>
	</div>

	{#if status}
		<p class="text-muted-foreground text-xs">{status}</p>
	{/if}
</div>
