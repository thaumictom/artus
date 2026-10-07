<script lang="ts">
	import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Switch from '$lib/components/Switch.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
	import Hotkeys from './Hotkeys.svelte';
	import AppThemeSetting from '../components/AppThemeSetting.svelte';

	let keybindDialogOpen = $state(false);
	const scrollbarTheme = 'os-theme-light';
</script>

<div class="flex flex-col">
	<AppThemeSetting />
	<CommonSetting
		title="Full-width content"
		description="Use all available page width instead of centering content within a maximum width."
	>
		<Switch
			id="full-width-content"
			checked={config.full_width_content}
			onCheckedChange={(enabled) => {
				config.full_width_content = enabled;
				void updateSetting('full_width_content');
			}}
		/>
	</CommonSetting>
	<CommonSetting
		title="Hide to tray when closing the app"
		description="Keep Artus running in the tray when you close its window. Use the tray menu to restart or quit."
	>
		<Switch
			id="hide-to-tray-on-close"
			checked={config.hide_to_tray_on_close}
			onCheckedChange={(enabled) => {
				config.hide_to_tray_on_close = enabled;
				void updateSetting('hide_to_tray_on_close');
			}}
		/>
	</CommonSetting>
	<CommonSetting
		title="Keybinds"
		description="Configure screenshot, navigation, and inventory shortcuts."
	>
		<Button class="shrink-0" onclick={() => (keybindDialogOpen = true)}>Edit keybinds</Button>
	</CommonSetting>
</div>

{#snippet keybindTitle()}Edit keybinds{/snippet}
{#snippet keybindDescription()}Select a keybind to record a new shortcut.{/snippet}
{#snippet keybindClose()}<Button>Close</Button>{/snippet}

<Dialog
	bind:open={keybindDialogOpen}
	title={keybindTitle}
	description={keybindDescription}
	dialogClose={keybindClose}
	contentProps={{ class: 'grid-rows-[auto_minmax(0,1fr)_auto]' }}
>
	<OverlayScrollbarsComponent
		defer
		class="flex-1 mr-1.75 min-w-0 min-h-0"
		options={{ scrollbars: { theme: scrollbarTheme, autoHide: 'move' } }}
	>
		<div class="pr-4.25 pl-6">
			<Hotkeys />
		</div>
	</OverlayScrollbarsComponent>
</Dialog>
