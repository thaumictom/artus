<script lang="ts">
	import { Separator } from 'bits-ui';
	import { config } from '$lib/settings.svelte';
	import HotkeyCapture from '../components/HotkeyCapture.svelte';

	type Keybind = [keyof typeof config.hotkeys, string];
	const groups: { title: string; keybinds: Keybind[] }[] = [
		{
			title: 'Capture',
			keybinds: [
				['screenshot', 'Screenshot'],
				['screenshot_add_mastery', 'Screenshot + mark mastery items'],
			],
		},
		{
			title: 'Overlay navigation',
			keybinds: [
				['cycle', 'Cycle forward'],
				['cycle_back', 'Cycle backward'],
				['navigate_up', 'Navigate up'],
				['navigate_down', 'Navigate down'],
				['navigate_left', 'Navigate left'],
				['navigate_right', 'Navigate right'],
			],
		},
		{
			title: 'Inventory',
			keybinds: [
				['inventory_decrement', 'Remove one'],
				['inventory_increment', 'Add one'],
			],
		},
	];
</script>

<div class="flex flex-col gap-5 pb-2">
	{#each groups as group, index}
		{#if index > 0}<Separator.Root class="bg-border h-px" />{/if}
		<section aria-labelledby={`keybind-group-${index}`}>
			<h3 id={`keybind-group-${index}`} class="mb-3 font-semibold text-muted-foreground text-xs uppercase tracking-wider">
				{group.title}
			</h3>
			<div class="grid grid-cols-2 gap-x-4 gap-y-4">
				{#each group.keybinds as [action, label]}
					<div class="min-w-0">
						<p class="mb-1 text-sm">{label}</p>
						<HotkeyCapture tauriHotkey={action} />
					</div>
				{/each}
			</div>
		</section>
	{/each}
</div>
