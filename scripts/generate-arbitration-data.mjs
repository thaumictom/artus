import { writeFileSync } from 'node:fs';

const output = new URL('../src/lib/data/', import.meta.url);
const download = async (url) => {
	const response = await fetch(url);
	if (!response.ok) throw new Error(`${url}: HTTP ${response.status}`);
	return response.text();
};
const [scheduleText, regionsText, dictText, factionsText, tierText] = await Promise.all([
	download('https://browse.wf/arbys.txt'),
	download('https://browse.wf/warframe-public-export-plus/ExportRegions.json'),
	download('https://browse.wf/warframe-public-export-plus/dict.en.json'),
	download('https://browse.wf/warframe-public-export-plus/ExportFactions.json'),
	download('https://raw.githubusercontent.com/calamity-inc/browse.wf/senpai/supplemental-data/arbyTiers.js'),
]);
const cutoff = Date.parse('2026-10-01T00:00:00Z') / 1000;
const rows = scheduleText.trim().split(/\r?\n/)
	.filter((line) => Number(line.split(',')[0]) >= cutoff);
const regions = JSON.parse(regionsText);
const dict = JSON.parse(dictText);
const factions = JSON.parse(factionsText);
const tiers = Object.fromEntries(
	[...tierText.matchAll(/\b((?:Sol|Clan|Settlement)Node\d+): "([A-S])"/g)]
		.map((match) => [match[1], match[2]]),
);
const nodes = Object.fromEntries([...new Set(rows.map((line) => line.split(',')[1]))].sort().map((key) => {
	const region = regions[key];
	return [key, {
		node: dict[region.name],
		planet: dict[region.systemName],
		mission: dict[region.missionName].toLowerCase().replace(/\b\w/g, (letter) => letter.toUpperCase()),
		faction: region.faction === 'FC_OROKIN' ? 'Corrupted'
			: region.faction === 'FC_MITW' ? 'The Murmur' : dict[factions[region.faction].name],
		tier: tiers[key] ?? 'F',
	}];
}));
writeFileSync(new URL('arbys.txt', output), `${rows.join('\n')}\n`);
writeFileSync(new URL('arbitration-nodes.json', output), `${JSON.stringify(nodes, null, 2)}\n`);
console.log(`${rows.length} schedule entries and ${Object.keys(nodes).length} node labels`);
