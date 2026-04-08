<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';

	import * as Select from '$lib/components/ui/select/index.js';

	type OcrDebugImagePayload = {
		png_bytes: number[];
		width: number;
		height: number;
		upscale_amount: number;
	};

	type OcrTextPayload = {
		text: string;
	};

	type OcrThemeOption = {
		name: string;
		rgb: [number, number, number];
	};

	type OcrThemeSettingsPayload = {
		themes: OcrThemeOption[];
		selected_theme: string;
	};

	let hotkey = $state('Home');
	let hotkeyStatus = $state('');
	let themeStatus = $state('');
	let ocrThemes = $state<OcrThemeOption[]>([]);
	let selectedOcrTheme = $state('EQUINOX');
	let isThemeInitialized = $state(false);
	let debugImageUrl = $state('');
	let debugImageInfo = $state('');
	let ocrText = $state('');
	let isImageFullscreen = $state(false);

	onMount(() => {
		let unlistenImage: (() => void) | undefined;
		let unlistenText: (() => void) | undefined;
		let activeUrl = '';

		listen<OcrDebugImagePayload>('ocr_debug_image', (event) => {
			const payload = event.payload;
			const bytes = new Uint8Array(payload.png_bytes);
			const blob = new Blob([bytes], { type: 'image/png' });
			const nextUrl = URL.createObjectURL(blob);
			if (activeUrl) URL.revokeObjectURL(activeUrl);
			activeUrl = nextUrl;
			debugImageUrl = nextUrl;
			debugImageInfo = `${payload.width}x${payload.height} (upscale ${payload.upscale_amount}x)`;
		}).then((cleanup) => {
			unlistenImage = cleanup;
		});

		listen<OcrTextPayload>('ocr_text_result', (event) => {
			ocrText = event.payload?.text ?? '';
		}).then((cleanup) => {
			unlistenText = cleanup;
		});

		(async () => {
			try {
				const [savedHotkey, themeSettings] = await Promise.all([
					invoke<string>('get_hotkey'),
					invoke<OcrThemeSettingsPayload>('get_ocr_theme_settings'),
				]);

				hotkey = savedHotkey;
				ocrThemes = themeSettings.themes;
				selectedOcrTheme = themeSettings.selected_theme;
				isThemeInitialized = true;
			} catch (error) {
				themeStatus = String(error);
			}
		})();

		return () => {
			unlistenImage?.();
			unlistenText?.();
			if (activeUrl) URL.revokeObjectURL(activeUrl);
		};
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

	function openImageFullscreen() {
		isImageFullscreen = true;
	}

	function closeImageFullscreen() {
		isImageFullscreen = false;
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

	{#if debugImageUrl}
		<div class="mt-4">
			<p class="text-sm">OCR Input ({debugImageInfo})</p>
			<button
				type="button"
				class="bg-transparent mt-2 p-0 border rounded w-full cursor-zoom-in"
				onclick={openImageFullscreen}
			>
				<img src={debugImageUrl} alt="OCR debug input" class="w-full" />
			</button>
		</div>
	{/if}

	{#if ocrText}
		<div class="mt-4">
			<p class="text-sm">OCR Text</p>
			<textarea
				class="mt-2 p-2 border rounded w-full h-40 text-sm"
				readonly
				value={ocrText}
			></textarea>
		</div>
	{/if}
</section>

{#if isImageFullscreen && debugImageUrl}
	<div
		class="z-50 fixed inset-0 flex justify-center items-center bg-black/90 p-4"
		role="button"
		tabindex="0"
		onclick={closeImageFullscreen}
		onkeydown={(e) =>
			e.key === 'Enter' || e.key === ' ' || e.key === 'Escape' ? closeImageFullscreen() : null}
	>
		<button
			class="top-4 right-4 absolute bg-white px-3 py-1 border rounded text-black"
			onclick={closeImageFullscreen}
		>
			Close
		</button>
		<div
			role="button"
			tabindex="0"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
		>
			<img
				src={debugImageUrl}
				alt="OCR debug input fullscreen"
				class="max-w-full max-h-full object-none"
			/>
		</div>
	</div>
{/if}
