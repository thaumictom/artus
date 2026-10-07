<script lang="ts">
	import { onDestroy } from 'svelte';
	import ColorPicker from 'svelte-awesome-color-picker';
	import { captureCustomTheme, customThemeColors, isHexColor, type CustomThemeColor } from '$lib/custom-theme';
	import { config } from '$lib/settings.svelte';
	import Select from '$lib/components/Select.svelte';
	import Button from '$lib/components/Button.svelte';
	import PalettePreview from '$lib/components/PalettePreview.svelte';

	let { onsave }: { onsave: () => Promise<void> } = $props();
	let selected = $state<CustomThemeColor>('background');
	let resetVersion = $state(0);
	let pendingSave: ReturnType<typeof setTimeout> | undefined;
	let palette = $derived(config.app_theme_custom_colors?.[config.app_theme_mode]);
	let items = $derived(customThemeColors.map(({ value, label }) => ({
		value, label, swatchColors: [palette?.[value] ?? '#000000', palette?.[value] ?? '#000000'] as [string, string],
	})));

	function changeColor(hex: string | null) {
		if (!isHexColor(hex) || !palette || palette[selected] === hex) return;
		palette[selected] = hex;
		clearTimeout(pendingSave);
		// Keep dragging responsive without writing settings for every pointer movement.
		pendingSave = setTimeout(() => { pendingSave = undefined; void onsave(); }, 300);
	}
	async function resetColors() {
		clearTimeout(pendingSave);
		pendingSave = undefined;
		config.app_theme_custom_colors = captureCustomTheme(config.app_theme, config.app_theme_accent, config.app_theme_accent_shade);
		resetVersion += 1;
		await onsave();
	}
	onDestroy(() => {
		if (pendingSave !== undefined) { clearTimeout(pendingSave); void onsave(); }
	});
</script>

{#if palette}
	<div class="grid gap-6 sm:grid-cols-[minmax(0,1fr)_auto]">
		<div class="flex flex-col gap-3">
			<label for="custom-theme-color" class="font-semibold">{config.app_theme_mode === 'dark' ? 'Dark' : 'Light'} palette</label>
			<Select type="single" {items} bind:value={selected} placeholder="Theme color" triggerProps={{ id: 'custom-theme-color' }} />
			<p class="max-w-80 text-sm text-muted-foreground">Choose a color to edit. Use Appearance above to customize the other light/dark palette.</p>
			<figure class="flex flex-col gap-2">
				<PalettePreview
					theme={config.app_theme}
					mode={config.app_theme_mode}
					accent={config.app_theme_accent}
					accentShade={config.app_theme_accent_shade}
					colors={palette}
					class="max-w-64"
				/>
				<figcaption class="text-sm text-muted-foreground">Live preview</figcaption>
			</figure>
			<Button class="self-start" onclick={resetColors} title="Restore both light and dark custom palettes from your selected theme">Reset custom theme</Button>
		</div>
		<div class="custom-color-picker" aria-label={`Edit ${items.find(({ value }) => value === selected)?.label}`}>
			{#key `${config.app_theme_mode}-${selected}-${resetVersion}`}
				<ColorPicker hex={palette[selected]} isDialog={false} isAlpha={false} textInputModes={['hex', 'rgb']} onInput={({ hex }) => changeColor(hex)} />
			{/key}
		</div>
	</div>
{/if}

<style>
	.custom-color-picker {
		--cp-bg-color: var(--theme-surface);
		--cp-border-color: var(--theme-muted);
		--cp-text-color: var(--theme-foreground);
		--cp-input-color: var(--theme-elevated);
		--cp-button-hover-color: var(--theme-muted);
		--focus-color: var(--theme-accent);
		--picker-width: 180px;
		--picker-height: 160px;
		--slider-width: 24px;
	}
	.custom-color-picker :global(.wrapper) { margin: 0; }
	.custom-color-picker :global(*) { border-radius: 0 !important; }
</style>
