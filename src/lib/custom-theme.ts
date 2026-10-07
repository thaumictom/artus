import type { AppTheme, AccentColor, AccentShade, ThemeMode } from './app-themes';

export const customThemeColors = [
	{ value: 'background', label: 'Background' },
	{ value: 'foreground', label: 'Text' },
	{ value: 'surface', label: 'Surface' },
	{ value: 'surface-foreground', label: 'Surface text' },
	{ value: 'elevated', label: 'Elevated surface' },
	{ value: 'elevated-foreground', label: 'Elevated text' },
	{ value: 'muted', label: 'Borders' },
	{ value: 'muted-foreground', label: 'Muted text' },
	{ value: 'accent', label: 'Accent' },
	{ value: 'accent-foreground', label: 'Accent text' },
] as const;
export type CustomThemeColor = (typeof customThemeColors)[number]['value'];
export type CustomPalette = Record<CustomThemeColor, string>;
export type CustomTheme = Record<ThemeMode, CustomPalette>;

export function isHexColor(value: unknown): value is string {
	return typeof value === 'string' && /^#[0-9a-f]{6}$/i.test(value);
}

export function isCustomTheme(value: unknown): value is CustomTheme {
	if (!value || typeof value !== 'object') return false;
	return (['light', 'dark'] as const).every((mode) => {
		const palette = (value as CustomTheme)[mode];
		return palette && customThemeColors.every(({ value: key }) => isHexColor(palette[key]));
	});
}

// Capture both preset appearances once; canvas converts CSS oklch colors to picker hex values.
export function captureCustomTheme(theme: AppTheme, accent: AccentColor, shade: AccentShade): CustomTheme {
	const sample = document.createElement('div');
	sample.className = `theme-${theme} theme-accent-${accent}`;
	sample.dataset.accentShade = shade;
	sample.style.display = 'none';
	document.body.append(sample);
	const context = document.createElement('canvas').getContext('2d', { willReadFrequently: true })!;
	try {
		return Object.fromEntries((['light', 'dark'] as const).map((mode) => {
			sample.dataset.themeMode = mode;
			const palette = Object.fromEntries(customThemeColors.map(({ value }) => {
				sample.style.color = `var(--theme-${value})`;
				context.fillStyle = getComputedStyle(sample).color;
				context.fillRect(0, 0, 1, 1);
				const [r, g, b] = context.getImageData(0, 0, 1, 1).data;
				return [value, '#' + [r, g, b].map((channel) => channel.toString(16).padStart(2, '0')).join('')];
			}));
			return [mode, palette];
		})) as CustomTheme;
	} finally {
		sample.remove();
	}
}
