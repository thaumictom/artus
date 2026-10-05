export const factionColors = {
	corpus: '#5B83B8',
	grineer: '#8FA075',
} as const;

export function factionColor(faction: string): string | undefined {
	return factionColors[faction.trim().toLowerCase() as keyof typeof factionColors];
}
