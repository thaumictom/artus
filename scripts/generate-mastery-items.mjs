import { writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import Items from '@wfcd/items';

const outputPath = resolve(process.cwd(), 'src/lib/data/masterable-items.json');

const masterableItems = new Items({ ignoreEnemies: true })
	.filter((item) => item.masterable)
	.map((item) => ({
		uniqueName: item.uniqueName,
		name: item.name,
		category: item.category ?? item.type ?? 'Unknown',
		masteryReq: typeof item.masteryReq === 'number' ? item.masteryReq : null
	}));

writeFileSync(outputPath, `${JSON.stringify(masterableItems, null, 2)}\n`, 'utf8');

console.log(`Generated ${masterableItems.length} masterable items at ${outputPath}`);
