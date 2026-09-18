import { writeFileSync } from 'node:fs';
import data from 'warframe-worldstate-data';

// Bundle only the English mappings; the upstream loader requires Node and loads every locale.
const labels = {
	languages: Object.fromEntries(Object.entries(data.languages).map(([key, value]) => [key.toLowerCase(), value])),
	nodes: data.solNodes,
	missions: data.missionTypes,
	factions: data.factions,
	fissures: data.fissureModifiers,
	syndicates: data.syndicates,
	sortie: data.sortie,
};
writeFileSync(new URL('../src/lib/data/worldstate-labels.json', import.meta.url), `${JSON.stringify(labels)}\n`);
console.log('Generated English world-state labels from warframe-worldstate-data (MIT).');
