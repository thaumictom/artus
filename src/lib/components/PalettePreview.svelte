<script lang="ts">
	import type { AccentColor, AccentShade, AppTheme, ThemeMode } from '$lib/app-themes';
	import { customThemeColors, type CustomPalette } from '$lib/custom-theme';
	import { cn } from '$lib/utils';

	let {
		theme,
		mode = 'dark',
		accent = 'emerald',
		accentShade = '500',
		colors,
		class: className,
	}: {
		theme: AppTheme;
		mode?: ThemeMode;
		accent?: AccentColor;
		accentShade?: AccentShade;
		colors?: CustomPalette;
		class?: string;
	} = $props();
	let customStyle = $derived(colors
		? customThemeColors.map(({ value }) => `--theme-${value}: ${colors[value]}`).join(';')
		: undefined);
</script>

<!-- Use the same palette scopes as the window so previews cannot drift from the theme. -->
<svg
	aria-hidden="true"
	viewBox="0 0 160 88"
	data-theme-mode={mode}
	data-accent-shade={accentShade}
	style={customStyle}
	class={cn(
		`theme-${theme} theme-accent-${accent}`,
		'block w-full max-w-40 overflow-hidden border border-muted bg-background',
		className,
	)}
>
	<!-- Title bar and sidebar. -->
	<path d="M0 0h160v18H0z M0 18h33v70H0z" class="fill-surface" />
	<!-- Panel dividers. -->
	<path d="M0 18h160 M33 18v70" class="stroke-muted" fill="none" />
	<!-- Active window dot. -->
	<path d="M5 6h12v5H5z" class="fill-accent" />
	<!-- Inactive window dots. -->
	<path d="M20 6h12v5H20z M35 6h12v5H35z" class="fill-muted" />
	<!-- Selected sidebar item. -->
	<path d="M5 33h22v5H5z" class="fill-accent" />
	<!-- Other sidebar items. -->
	<path d="M5 25h22v5H5z M5 41h22v5H5z M5 49h22v5H5z" class="fill-elevated" />
	<!-- Content heading. -->
	<path d="M44 30h80v6H44z" class="fill-foreground" />
	<!-- Secondary text. -->
	<path d="M44 40h100v4H44z M44 48h60v4H44z" class="fill-muted-foreground" />
	<!-- Content separator. -->
	<path d="M44 56h100v2H44z" class="fill-elevated" />
	<!-- Accent action. -->
	<path d="M44 62h50v6H44z" class="fill-accent" />
</svg>
