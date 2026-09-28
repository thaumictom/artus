const categoryTags: [tag: string, category: string][] = [
	['relic', 'Relics'],
	['mod', 'Mods'],
	['arcane_enhancement', 'Arcanes'],
	['arcane_helmet', 'Arcanes'],
	['warframe', 'Warframes'],
	['weapon', 'Weapons'],
	['companion', 'Companions'],
	['sentinel', 'Companions'],
	['kubrow', 'Companions'],
	['kavat', 'Companions'],
	['hound', 'Companions'],
	['moa', 'Companions'],
	['archwing', 'Archwing'],
	['necramech', 'Necramechs'],
	['railjack', 'Railjack'],
	['fish', 'Resources'],
	['scene', 'Scenes'],
	['skin', 'Cosmetics'],
	['emote', 'Cosmetics'],
	['blueprint', 'Blueprints'],
	['set', 'Sets'],
	['component', 'Components'],
	['misc', 'Miscellaneous'],
];

export function wfmCategory(tags: string[]): string {
	return categoryTags.find(([tag]) => tags.includes(tag))?.[1] ?? 'Other';
}

export function formatWfmTag(tag: string): string {
	if (tag === 'pvp') return 'PvP';
	if (tag === 'k_drive') return 'K-Drive';
	return tag.replaceAll('_', ' ').replace(/\b\w/g, (letter) => letter.toUpperCase());
}
