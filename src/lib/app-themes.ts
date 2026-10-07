// Values match the theme classes in variants.css.
export const appThemes = [
	{ value: 'default', label: 'Default', swatchColors: ['var(--color-mist-600)', 'var(--color-emerald-500)'] },
	{ value: 'stalker', label: 'Stalker', swatchColors: ['color-mix(in oklch, color-mix(in oklch, oklch(from var(--color-red-700) l 0 h), oklch(from var(--color-red-700) l calc(c * 2) h) var(--theme-desature-700)), black var(--theme-darken-700))', 'color-mix(in oklch, color-mix(in oklch, oklch(from var(--color-red-500) l 0 h), oklch(from var(--color-red-500) l calc(c * 2) h) var(--theme-desature-500)), black var(--theme-darken-500))'] },
	{ value: 'high-contrast', label: 'High Contrast', swatchColors: ['color-mix(in oklch, color-mix(in oklch, oklch(from var(--color-sky-700) l 0 h), oklch(from var(--color-sky-700) l calc(c * 2) h) var(--theme-desature-700)), black var(--theme-darken-700))', 'color-mix(in oklch, color-mix(in oklch, oklch(from var(--color-yellow-300) l 0 h), oklch(from var(--color-yellow-300) l calc(c * 2) h) var(--theme-desature-300)), black var(--theme-darken-300))'] },
	{ value: 'baruuk', label: 'Baruuk', swatchColors: ['color-mix(in oklch, color-mix(in oklch, oklch(from var(--color-amber-700) l 0 h), oklch(from var(--color-amber-700) l calc(c * 2) h) var(--theme-desature-700)), black var(--theme-darken-700))', 'color-mix(in oklch, color-mix(in oklch, oklch(from var(--color-amber-400) l 0 h), oklch(from var(--color-amber-400) l calc(c * 2) h) var(--theme-desature-400)), black var(--theme-darken-400))'] },
	{ value: 'fortuna', label: 'Fortuna', swatchColors: ['color-mix(in oklch, color-mix(in oklch, oklch(from var(--color-blue-700) l 0 h), oklch(from var(--color-blue-700) l calc(c * 2) h) var(--theme-desature-700)), black var(--theme-darken-700))', 'color-mix(in oklch, color-mix(in oklch, oklch(from var(--color-fuchsia-400) l 0 h), oklch(from var(--color-fuchsia-400) l calc(c * 2) h) var(--theme-desature-400)), black var(--theme-darken-400))'] },
	{ value: 'corpus', label: 'Corpus', swatchColors: ['color-mix(in oklch, color-mix(in oklch, oklch(from var(--color-cyan-700) l 0 h), oklch(from var(--color-cyan-700) l calc(c * 2) h) var(--theme-desature-700)), black var(--theme-darken-700))', 'color-mix(in oklch, color-mix(in oklch, oklch(from var(--color-cyan-400) l 0 h), oklch(from var(--color-cyan-400) l calc(c * 2) h) var(--theme-desature-400)), black var(--theme-darken-400))'] },
	{ value: 'slate', label: 'Slate', swatchColors: ['var(--color-slate-600)', 'var(--color-emerald-500)'] },
	{ value: 'gray', label: 'Gray', swatchColors: ['var(--color-gray-600)', 'var(--color-emerald-500)'] },
	{ value: 'zinc', label: 'Zinc', swatchColors: ['var(--color-zinc-600)', 'var(--color-emerald-500)'] },
	{ value: 'neutral', label: 'Neutral', swatchColors: ['var(--color-neutral-600)', 'var(--color-emerald-500)'] },
	{ value: 'stone', label: 'Stone', swatchColors: ['var(--color-stone-600)', 'var(--color-emerald-500)'] },
	{ value: 'taupe', label: 'Taupe', swatchColors: ['var(--color-taupe-600)', 'var(--color-emerald-500)'] },
	{ value: 'mauve', label: 'Mauve', swatchColors: ['var(--color-mauve-600)', 'var(--color-emerald-500)'] },
	{ value: 'olive', label: 'Olive', swatchColors: ['var(--color-olive-600)', 'var(--color-emerald-500)'] },
] as const;

export type AppTheme = (typeof appThemes)[number]['value'];
export function isAppTheme(value: unknown): value is AppTheme {
	return typeof value === 'string' && appThemes.some((theme) => theme.value === value);
}
