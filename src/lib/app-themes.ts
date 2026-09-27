// Values match the theme classes in variants.css.
export const appThemes = [
  { value: 'default', label: 'Default' },
  { value: 'vitruvian', label: 'Vitruvian' },
  { value: 'stalker', label: 'Stalker' },
  { value: 'baruuk', label: 'Baruuk' },
  { value: 'corpus', label: 'Corpus' },
  { value: 'fortuna', label: 'Fortuna' },
  { value: 'grineer', label: 'Grineer' },
  { value: 'lotus', label: 'Lotus' },
  { value: 'nidus', label: 'Nidus' },
  { value: 'orokin', label: 'Orokin' },
  { value: 'tenno', label: 'Tenno' },
  { value: 'high-contrast', label: 'High Contrast' },
  { value: 'legacy', label: 'Legacy' },
  { value: 'equinox', label: 'Equinox' },
  { value: 'dark-lotus', label: 'Dark Lotus' },
  { value: 'zephyr', label: 'Zephyr Harrier' },
  { value: 'lunar-renewal', label: 'Lunar Renewal' },
] as const;

export type AppTheme = (typeof appThemes)[number]['value'];
export function isAppTheme(value: unknown): value is AppTheme {
  return typeof value === 'string' && appThemes.some((theme) => theme.value === value);
}
