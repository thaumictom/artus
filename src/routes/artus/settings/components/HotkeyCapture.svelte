<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { Button } from 'bits-ui';
	import { onMount } from 'svelte';

	let isRecording = $state(false);
	let isLoading = $state(true);
	let isSaving = $state(false);
	let keybind: string | null = $state(null);
	let status: string | null = $state(null);

	let { tauriHotkey }: { tauriHotkey: string } = $props();

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

	async function loadKeybind() {
		isLoading = true;
		status = null;

		try {
			keybind = await invoke<string>('get_hotkey', { action: tauriHotkey });
		} catch (error) {
			status = String(error);
		} finally {
			isLoading = false;
		}
	}

	async function saveKeybind(nextKeybind: string) {
		isSaving = true;
		status = null;

		try {
			keybind = await invoke<string>('set_hotkey', {
				action: tauriHotkey,
				hotkey: nextKeybind,
			});
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
		if (isLoading || isSaving) return;
		status = null;
		isRecording = !isRecording;
	}

	onMount(() => {
		void loadKeybind();
	});
</script>

<svelte:window onkeydown={handleKeyDown} />

<Button.Root
	onclick={toggleRecording}
	disabled={isLoading || isSaving}
	tabindex={-1}
	class="group flex justify-between items-center gap-1 p-0.5 border outline-0 min-w-80 cursor-pointer"
>
	<div class="p-1.5">
		{#if isLoading}
			<span class="text-muted-foreground">Loading...</span>
		{:else if keybind}
			<span class="font-condensed font-bold text-sm">{formatKeybindForDisplay(keybind)}</span>
		{:else}
			<span class="text-muted-foreground">No keybind set</span>
		{/if}
	</div>
	<div class="bg-surface group-hover:bg-muted p-1.5 transition">
		{#if isSaving}
			Saving...
		{:else if !isRecording && !keybind}
			Set keybind
		{:else if !isRecording && keybind}
			Edit keybind
		{:else}
			<span class="text-danger">Stop recording</span>
		{/if}
	</div>
</Button.Root>

<!-- {#if status}
	<p class="mt-1 text-muted-foreground text-xs">{status}</p>
{/if} -->
