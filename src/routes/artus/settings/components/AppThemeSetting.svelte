<script lang="ts">
	import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
	import {
		accentColors,
		accentShades,
		appThemes,
		isSpecialtyTheme,
		neutralThemes,
		specialtyThemes,
		themeModes,
		type AppTheme,
	} from '$lib/app-themes';
	import { config, updateSetting } from '$lib/settings.svelte';
	import Button from '$lib/components/Button.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import ColorSwatch from '$lib/components/ColorSwatch.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import PalettePreview from '$lib/components/PalettePreview.svelte';
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import Icon from '@iconify/svelte';
	import Switch from '$lib/components/Switch.svelte';
	import CustomThemeEditor from './CustomThemeEditor.svelte';
	import { captureCustomTheme } from '$lib/custom-theme';

	let open = $state(false);
	let saveError = $state(false);
	let specialty = $derived(!config.app_theme_custom_enabled && isSpecialtyTheme(config.app_theme));
	let themeLabel = $derived(
		config.app_theme_custom_enabled
			? 'Custom'
			: appThemes.find(({ value }) => value === config.app_theme)?.label,
	);
	const swatches = accentColors.map((value) => ({
		value,
		label: value[0].toUpperCase() + value.slice(1),
	}));

	async function save(
		key:
			| 'app_theme'
			| 'app_theme_mode'
			| 'app_theme_accent'
			| 'app_theme_accent_shade'
			| 'app_theme_custom_enabled'
			| 'app_theme_custom_colors'
			| 'overlay_opposite_theme',
	) {
		try {
			await updateSetting(key);
			saveError = false;
		} catch (error) {
			console.error('Could not save app theme:', error);
			saveError = true;
		}
	}
</script>

<CommonSetting title="App theme" description="Choose your palette, appearance, and accent color.">
	<Button
		variant="default"
		size="none"
		class="inline-flex items-center min-w-52 min-h-11 transition-colors shrink-0"
		aria-label={`Edit app theme: ${themeLabel}`}
		aria-haspopup="dialog"
		aria-expanded={open}
		onclick={() => (open = true)}
	>
		<span class="flex flex-1 items-center gap-2.5 px-3 py-1.5">
			<span
				aria-hidden="true"
				data-theme-mode={config.app_theme_mode}
				data-accent-shade={config.app_theme_accent_shade}
				class={`theme-${config.app_theme} theme-accent-${config.app_theme_accent} inline-flex shrink-0`}
			>
				<ColorSwatch
					colors={config.app_theme_custom_enabled && config.app_theme_custom_colors
						? [
								config.app_theme_custom_colors[config.app_theme_mode].muted,
								config.app_theme_custom_colors[config.app_theme_mode].accent,
							]
						: ['var(--theme-muted)', 'var(--theme-accent)']}
				/>
			</span>
			<span class="whitespace-nowrap">{themeLabel}</span>
		</span>
		<span
			aria-hidden="true"
			class="flex items-center self-stretch px-4 border-l font-semibold text-xs uppercase tracking-wider edit"
		>
			Edit
			<Icon icon="material-symbols:chevron-right-rounded" class="-mr-1 size-4" />
		</span>
	</Button>
</CommonSetting>

{#snippet title()}App theme{/snippet}
{#snippet description()}Choose your palette, appearance, and accent color{/snippet}
{#snippet dialogClose()}<Button>Done</Button>{/snippet}
{#snippet themeOption(option: { value: AppTheme; label: string })}
	<PalettePreview
		theme={option.value}
		mode={config.app_theme_mode}
		accent={config.app_theme_accent}
		accentShade={config.app_theme_accent_shade}
	/>
	<span>{option.label}</span>
{/snippet}
{#snippet accentOption(option: { value: string; label: string })}
	<span
		data-accent-shade={config.app_theme_accent_shade}
		class={`theme-accent-${option.value} flex size-6 items-center justify-center rounded-full bg-(--app-neutral-accent)`}
	>
		{#if config.app_theme_accent === option.value}
			<svg
				aria-hidden="true"
				viewBox="0 0 24 24"
				class="size-4 text-black/80"
				fill="none"
				stroke="currentColor"
				stroke-width="2.5"
			>
				<path d="m5 12 4 4L19 6" />
			</svg>
		{/if}
	</span>
{/snippet}

<Dialog
	bind:open
	{title}
	{description}
	{dialogClose}
	contentProps={{
		class:
			'w-[min(60rem,calc(100vw-1.5rem))] h-[min(52rem,calc(100vh-1.5rem))] grid-rows-[auto_minmax(0,1fr)_auto]',
	}}
>
	<OverlayScrollbarsComponent
		defer
		class="mr-1.75 min-w-0 min-h-0"
		options={{
			scrollbars: {
				theme: specialty || config.app_theme_mode === 'dark' ? 'os-theme-light' : 'os-theme-dark',
				autoHide: 'move',
			},
		}}
	>
		<div class="flex flex-col gap-6 pr-4.25 pb-1 pl-6">
			{#if !config.app_theme_custom_enabled}
				<section class="flex flex-col gap-3" aria-labelledby="theme-defaults">
					<h3 id="theme-defaults" class="font-semibold text-lg">Default themes</h3>
					<RadioGroup
						options={neutralThemes}
						label="Default themes"
						variant="cards"
						class="gap-4 grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5"
						itemClass="gap-3 p-4 text-base"
						value={specialty ? '' : config.app_theme}
						onValueChange={(value) => {
							config.app_theme = value as AppTheme;
							void save('app_theme');
						}}
						optionContent={themeOption}
					/>
				</section>
			{/if}

			<div
				class={config.app_theme_custom_enabled
					? 'grid gap-4'
					: 'gap-4 grid md:grid-cols-[minmax(0,2fr)_minmax(0,4fr)] py-6 border-y'}
			>
				<section
					class={`flex flex-col gap-4 min-w-0 ${config.app_theme_custom_enabled ? '' : 'md:pr-6 md:border-r'}`}
					aria-labelledby="theme-appearance"
				>
					<h3 id="theme-appearance" class="font-semibold text-lg">Appearance</h3>
					<div class="flex flex-col gap-5">
						<RadioGroup
							options={themeModes}
							label="Appearance"
							variant="tabs"
							itemClass="min-w-20 px-4 py-2"
							class={specialty ? 'grayscale opacity-50' : ''}
							bind:value={config.app_theme_mode}
							disabled={specialty}
							onValueChange={() => void save('app_theme_mode')}
						/>
						<div class={`flex items-start gap-3 ${specialty ? 'grayscale opacity-50' : ''}`}>
							<Checkbox
								id="overlay-opposite-theme"
								class="mt-0.5 size-5"
								checked={config.overlay_opposite_theme}
								disabled={specialty}
								onCheckedChange={(checked) => {
									config.overlay_opposite_theme = checked;
									void save('overlay_opposite_theme');
								}}
							/>
							<div>
								<label
									for="overlay-opposite-theme"
									class={specialty ? 'cursor-not-allowed' : 'cursor-pointer'}
								>
									Use opposite theme ingame
								</label>
							</div>
						</div>
					</div>
				</section>

				{#if !config.app_theme_custom_enabled}
					<section
						class="flex flex-col gap-4 pt-6 md:pt-0 border-t md:border-t-0 min-w-0"
						aria-labelledby="theme-accent"
					>
						<h3 id="theme-accent" class="flex items-baseline gap-3 font-semibold text-lg">
							Accent color
							<span class="font-normal text-muted-foreground text-sm">
								{specialty
									? 'Fixed by theme'
									: swatches.find(({ value }) => value === config.app_theme_accent)?.label}
							</span>
						</h3>
						<RadioGroup
							options={swatches}
							label="Accent color"
							variant="swatches"
							class={`gap-1 ${specialty ? 'grayscale opacity-50' : ''}`}
							bind:value={config.app_theme_accent}
							disabled={specialty}
							onValueChange={() => void save('app_theme_accent')}
							optionContent={accentOption}
						/>
						<RadioGroup
							options={accentShades}
							label="Accent shade"
							variant="tabs"
							itemClass="min-w-24 px-4 py-2"
							class={specialty ? 'grayscale opacity-50' : ''}
							bind:value={config.app_theme_accent_shade}
							disabled={specialty}
							onValueChange={() => void save('app_theme_accent_shade')}
						/>
					</section>
				{/if}
			</div>

			{#if !config.app_theme_custom_enabled}
				<section class="flex flex-col gap-3" aria-labelledby="theme-warframe">
					<div>
						<h3 id="theme-warframe" class="font-semibold text-lg">Warframe themes</h3>
						<p class="text-muted-foreground text-sm">Fixed appearance and accent.</p>
					</div>
					<RadioGroup
						options={specialtyThemes}
						label="Warframe themes"
						variant="cards"
						class="gap-4 grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5"
						itemClass="gap-3 p-4 text-base"
						value={specialty ? config.app_theme : ''}
						onValueChange={(value) => {
							config.app_theme = value as AppTheme;
							void save('app_theme');
						}}
						optionContent={themeOption}
					/>
				</section>
			{/if}
			<section class="flex flex-col gap-5 pt-6 border-t" aria-labelledby="theme-custom">
				<div class="flex justify-between items-center gap-4">
					<div>
						<label
							id="theme-custom"
							for="custom-theme-mode"
							class="font-semibold text-lg cursor-pointer"
						>
							Custom theme mode (experimental)
						</label>
						<p id="theme-custom-description" class="text-muted-foreground text-sm">
							Customize individual colors. Turn off to return to your selected theme.
						</p>
					</div>
					<Switch
						id="custom-theme-mode"
						aria-describedby="theme-custom-description"
						checked={config.app_theme_custom_enabled}
						onCheckedChange={async (checked) => {
							if (checked && !config.app_theme_custom_colors) {
								config.app_theme_custom_colors = captureCustomTheme(
									config.app_theme,
									config.app_theme_accent,
									config.app_theme_accent_shade,
								);
								await save('app_theme_custom_colors');
							}
							config.app_theme_custom_enabled = checked;
							await save('app_theme_custom_enabled');
						}}
					/>
				</div>
				{#if config.app_theme_custom_enabled}
					<CustomThemeEditor onsave={() => save('app_theme_custom_colors')} />
				{/if}
			</section>
			{#if saveError}<p role="alert" class="text-danger text-sm">
					Could not save your theme. Choose an option again to retry.
				</p>{/if}
		</div>
	</OverlayScrollbarsComponent>
</Dialog>
