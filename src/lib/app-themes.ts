export const neutralThemes = [
	{ value: 'slate', label: 'Slate' },
	{ value: 'gray', label: 'Gray' },
	{ value: 'zinc', label: 'Zinc' },
	{ value: 'neutral', label: 'Neutral' },
	{ value: 'stone', label: 'Stone' },
	{ value: 'taupe', label: 'Taupe' },
	{ value: 'mauve', label: 'Mauve' },
	{ value: 'mist', label: 'Mist' },
	{ value: 'olive', label: 'Olive' },
] as const;

export const specialtyThemes = [
	{ value: 'stalker', label: 'Stalker' },
	{ value: 'high-contrast', label: 'High Contrast' },
	{ value: 'baruuk', label: 'Baruuk' },
	{ value: 'fortuna', label: 'Fortuna' },
	{ value: 'corpus', label: 'Corpus' },
] as const;

export const appThemes = [...neutralThemes, ...specialtyThemes] as const;
export const accentColors = [
	'red', 'orange', 'amber', 'yellow', 'lime', 'green', 'emerald', 'teal', 'cyan',
	'sky', 'blue', 'indigo', 'violet', 'purple', 'fuchsia', 'pink', 'rose',
] as const;
export const themeModes = [
	{ value: 'light', label: 'Light' },
	{ value: 'dark', label: 'Dark' },
] as const;
export const accentShades = [
	{ value: '400', label: 'Lighter' },
	{ value: '500', label: 'Default' },
	{ value: '600', label: 'Darker' },
] as const;

export type AppTheme = (typeof appThemes)[number]['value'];
export type ThemeMode = (typeof themeModes)[number]['value'];
export type AccentColor = (typeof accentColors)[number];
export type AccentShade = (typeof accentShades)[number]['value'];

export function isAppTheme(value: unknown): value is AppTheme {
	return typeof value === 'string' && appThemes.some((theme) => theme.value === value);
}
export function isThemeMode(value: unknown): value is ThemeMode {
	return value === 'light' || value === 'dark';
}
export function isAccentColor(value: unknown): value is AccentColor {
	return typeof value === 'string' && accentColors.some((color) => color === value);
}
export function isAccentShade(value: unknown): value is AccentShade {
	return accentShades.some((shade) => shade.value === value);
}
export function isSpecialtyTheme(value: AppTheme) {
	return specialtyThemes.some((theme) => theme.value === value);
}
