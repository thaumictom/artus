<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	import * as Select from '$lib/components/ui/select/index.js';

	type OcrThemeOption = {
		name: string;
		rgb: [number, number, number];
	};

	type OcrThemeSettingsPayload = {
		themes: OcrThemeOption[];
		selected_theme: string;
	};

	const MIN_OVERLAY_DURATION_SECS = 1;
	const MAX_OVERLAY_DURATION_SECS = 60;
	const commitHash = import.meta.env.VITE_ARTUS_COMMIT_HASH || 'unknown';

	let hotkey = $state('Home');
	let hotkeyStatus = $state('');
	let themeStatus = $state('');
	let overlayDurationStatus = $state('');
	let overlayModeStatus = $state('');
	let ocrThemes = $state<OcrThemeOption[]>([]);
	let selectedOcrTheme = $state('EQUINOX');
	let overlayDurationInput = $state(10);
	let overlayToggleMode = $state(false);
	let isThemeInitialized = $state(false);

	onMount(() => {
		(async () => {
			try {
				const [savedHotkey, themeSettings, overlayDurationSecs, savedToggleMode] =
					await Promise.all([
						invoke<string>('get_hotkey'),
						invoke<OcrThemeSettingsPayload>('get_ocr_theme_settings'),
						invoke<number>('get_overlay_duration_secs'),
						invoke<boolean>('get_overlay_toggle_mode'),
					]);

				hotkey = savedHotkey;
				ocrThemes = themeSettings.themes;
				selectedOcrTheme = themeSettings.selected_theme;
				overlayDurationInput = overlayDurationSecs;
				overlayToggleMode = savedToggleMode;
				isThemeInitialized = true;
			} catch (error) {
				themeStatus = String(error);
			}
		})();
	});

	async function saveHotkey() {
		hotkeyStatus = '';
		try {
			await invoke('set_hotkey', { hotkey });
			hotkeyStatus = 'Saved';
		} catch (error) {
			hotkeyStatus = String(error);
		}
	}

	async function saveOcrTheme(theme: string = selectedOcrTheme) {
		themeStatus = '';
		try {
			await invoke('set_ocr_theme', { theme });
			themeStatus = `OCR theme set to ${formatThemeName(theme)}`;
		} catch (error) {
			themeStatus = String(error);
		}
	}

	function handleOcrThemeChange(theme: string) {
		selectedOcrTheme = theme;
		if (!isThemeInitialized) {
			return;
		}
		void saveOcrTheme(theme);
	}

	async function saveOverlayMode(enabled: boolean = overlayToggleMode) {
		overlayModeStatus = '';

		try {
			const savedMode = await invoke<boolean>('set_overlay_toggle_mode', { enabled });
			overlayToggleMode = savedMode;
			overlayModeStatus = savedMode
				? 'Overlay mode set to Toggle (press hotkey again to hide).'
				: 'Overlay mode set to Timer.';
		} catch (error) {
			overlayModeStatus = String(error);
		}
	}

	async function saveOverlayDuration() {
		overlayDurationStatus = '';

		const parsed = Number(overlayDurationInput);
		const normalized = Number.isFinite(parsed) ? Math.trunc(parsed) : Number.NaN;
		if (
			!Number.isFinite(normalized) ||
			normalized < MIN_OVERLAY_DURATION_SECS ||
			normalized > MAX_OVERLAY_DURATION_SECS
		) {
			overlayDurationStatus = `Duration must be between ${MIN_OVERLAY_DURATION_SECS} and ${MAX_OVERLAY_DURATION_SECS} seconds.`;
			return;
		}

		try {
			const savedSeconds = await invoke<number>('set_overlay_duration_secs', {
				seconds: normalized,
			});
			overlayDurationInput = savedSeconds;
			overlayDurationStatus = `Overlay duration set to ${savedSeconds}s`;
		} catch (error) {
			overlayDurationStatus = String(error);
		}
	}

	function formatThemeName(theme: string): string {
		return theme
			.toLowerCase()
			.split('_')
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(' ');
	}

	function formatThemeOption(theme: OcrThemeOption): string {
		return `${formatThemeName(theme.name)} (${theme.rgb[0]}, ${theme.rgb[1]}, ${theme.rgb[2]})`;
	}

	function selectedThemeLabel(): string {
		const selected = ocrThemes.find((theme) => theme.name === selectedOcrTheme);
		if (!selected) {
			return formatThemeName(selectedOcrTheme);
		}
		return formatThemeOption(selected);
	}
</script>

<section class="max-w-md">
	<p class="text-sm">Set global shortcut (examples: Home, Ctrl+Shift+H).</p>

	<div class="flex items-center gap-2 mt-4">
		<input class="px-2 py-1 border rounded w-full" bind:value={hotkey} placeholder="Home" />
		<button class="px-3 py-1 border rounded" onclick={saveHotkey}>Save</button>
	</div>

	{#if hotkeyStatus}
		<p class="mt-2 text-sm">{hotkeyStatus}</p>
	{/if}

	<div class="mt-6">
		<p class="text-sm">OCR Theme (Primary)</p>
		<Select.Root
			type="single"
			bind:value={selectedOcrTheme}
			onValueChange={handleOcrThemeChange}
			items={ocrThemes.map((theme) => ({ value: theme.name, label: formatThemeName(theme.name) }))}
		>
			<Select.Trigger class="mt-2 w-full">{selectedThemeLabel()}</Select.Trigger>
			<Select.Content>
				{#each ocrThemes as theme (theme.name)}
					<Select.Item value={theme.name} label={formatThemeName(theme.name)}>
						{formatThemeOption(theme)}
					</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>
	</div>

	{#if themeStatus}
		<p class="mt-2 text-sm">{themeStatus}</p>
	{/if}

	<div class="mt-6 p-3 border rounded">
		<p class="font-medium text-sm">Overlay Mode</p>
		<label class="flex items-center gap-2 mt-2 text-sm cursor-pointer">
			<input
				type="checkbox"
				bind:checked={overlayToggleMode}
				onchange={() => void saveOverlayMode(overlayToggleMode)}
			/>
			<span>Toggle mode (press once to show, press again to hide)</span>
		</label>
		<p class="mt-1 text-xs">
			{overlayToggleMode
				? 'Toggle mode ignores the timer and keeps the overlay visible until the next hotkey press.'
				: 'Timer mode auto-hides the overlay after the configured duration.'}
		</p>
	</div>

	{#if overlayModeStatus}
		<p class="mt-2 text-sm">{overlayModeStatus}</p>
	{/if}

	<div class="mt-6">
		<p class="text-sm">Overlay Duration (seconds)</p>
		<div class="flex items-center gap-2 mt-2">
			<input
				type="number"
				min={MIN_OVERLAY_DURATION_SECS}
				max={MAX_OVERLAY_DURATION_SECS}
				step="1"
				class="px-2 py-1 border rounded w-full"
				disabled={overlayToggleMode}
				bind:value={overlayDurationInput}
			/>
			<button
				class="px-3 py-1 border rounded"
				disabled={overlayToggleMode}
				onclick={saveOverlayDuration}
			>
				Save
			</button>
		</div>
		<p class="mt-1 text-xs">
			Range: {MIN_OVERLAY_DURATION_SECS}-{MAX_OVERLAY_DURATION_SECS} seconds.
			{overlayToggleMode ? ' Disabled while Toggle mode is enabled.' : ''}
		</p>
	</div>

	{#if overlayDurationStatus}
		<p class="mt-2 text-sm">{overlayDurationStatus}</p>
	{/if}

	<div class="mt-6 pt-4 border-t">
		<p class="text-sm">Build Commit</p>
		<p class="font-mono text-muted-foreground text-xs break-all">{commitHash}</p>
	</div>
</section>
