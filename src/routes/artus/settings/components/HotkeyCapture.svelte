<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { Button } from 'bits-ui';
	import { config } from '$lib/settings.svelte';

	let isRecording = $state(false);
	let isSaving = $state(false);
	let status: string | null = $state(null);

	let { tauriHotkey }: { tauriHotkey: keyof typeof config.hotkeys } = $props();

	function normalizeCapturedKey(key: string): string {
		if (key === ' ') return 'space';
		return key;
	}

	function formatKeybindForDisplay(value: string): string {
		return value
			.split('+')
			.map((segment) => segment.trim().toUpperCase())
			.join(' + ');
	}

	function buildKeybind(event: KeyboardEvent): string | null {
		const key = event.key.toLowerCase();

		// If the user only pressed a modifier, wait for the next key.
		if (['control', 'shift', 'alt', 'meta'].includes(key)) {
			return null;
		}

		const keys: string[] = [];
		if (event.ctrlKey) keys.push('ctrl');
		if (event.altKey) keys.push('alt');
		if (event.shiftKey) keys.push('shift');
		if (event.metaKey) keys.push('super');

		keys.push(normalizeCapturedKey(key));
		return keys.join('+');
	}

	async function saveKeybind(nextKeybind: string) {
		isSaving = true;
		status = null;

		try {
			const savedKeybind = await invoke<string>('set_hotkey', {
				action: tauriHotkey,
				hotkey: nextKeybind,
			});
			config.hotkeys[tauriHotkey] = savedKeybind;
			status = 'Saved';
		} catch (error) {
			status = String(error);
		} finally {
			isSaving = false;
		}
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (!isRecording) return;

		event.preventDefault();

		const nextKeybind = buildKeybind(event);
		if (!nextKeybind) return;

		isRecording = false;
		void saveKeybind(nextKeybind);
	}

	function toggleRecording() {
		if (isSaving) return;
		status = null;
		isRecording = !isRecording;
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

<Button.Root
	onclick={toggleRecording}
	disabled={isSaving}
	tabindex={-1}
	class="group flex justify-between items-center gap-1 p-0.5 border outline-0 min-w-80 cursor-pointer"
>
	<div class="p-1.5">
		{#if config.hotkeys[tauriHotkey]}
			<span class="font-condensed font-medium">{formatKeybindForDisplay(config.hotkeys[tauriHotkey])}</span>
		{:else}
			<span class="text-muted-foreground">No keybind set</span>
		{/if}
	</div>
	<div class="bg-surface group-hover:bg-muted p-1.5 transition">
		{#if isSaving}
			Saving...
		{:else if !isRecording && !config.hotkeys[tauriHotkey]}
			Set keybind
		{:else if !isRecording && config.hotkeys[tauriHotkey]}
			Edit keybind
		{:else}
			<span class="text-danger">Stop recording</span>
		{/if}
	</div>
</Button.Root>
