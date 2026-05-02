import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const newVersion = process.argv[2];

if (!newVersion) {
	console.error('Please provide a version string. Usage: node scripts/update-version.mjs <version>');
	process.exit(1);
}

const rootDir = process.cwd();
const packageJsonPath = resolve(rootDir, 'package.json');
const tauriConfigPath = resolve(rootDir, 'src-tauri', 'tauri.conf.json');
const cargoTomlPath = resolve(rootDir, 'src-tauri', 'Cargo.toml');

function updateJson(filePath) {
	try {
		const content = JSON.parse(readFileSync(filePath, 'utf8'));
		if (content.version !== newVersion) {
			content.version = newVersion;
			writeFileSync(filePath, JSON.stringify(content, null, '\t') + '\n', 'utf8');
			console.log(`Updated version to ${newVersion} in ${filePath}`);
		} else {
			console.log(`Version already ${newVersion} in ${filePath}`);
		}
	} catch (error) {
		console.error(`Failed to update JSON at ${filePath}:`, error);
		process.exit(1);
	}
}

function updateCargoToml(filePath) {
	try {
		const content = readFileSync(filePath, 'utf8');
		const lines = content.split(/\r?\n/);
		let inPackageSection = false;
		let updated = false;

		for (let i = 0; i < lines.length; i++) {
			const line = lines[i].trim();

			if (line.startsWith('[') && line.endsWith(']')) {
				inPackageSection = line === '[package]';
				continue;
			}

			if (!inPackageSection) {
				continue;
			}

			const uncommentedLine = line.split('#', 1)[0].trim();
			if (uncommentedLine.match(/^version\s*=\s*["']([^"']+)["']\s*$/)) {
				const currentVersionMatch = uncommentedLine.match(/^version\s*=\s*["']([^"']+)["']\s*$/);
				if (currentVersionMatch && currentVersionMatch[1] === newVersion) {
					console.log(`Version already ${newVersion} in ${filePath}`);
					updated = true;
					break;
				}

				lines[i] = line.replace(/(version\s*=\s*["'])([^"']+)["']/, `$1${newVersion}"`);
				updated = true;
				console.log(`Updated version to ${newVersion} in ${filePath}`);
				break;
			}
		}

		if (!updated) {
			console.error(`Could not find version field in [package] section of ${filePath}`);
			process.exit(1);
		}

		writeFileSync(filePath, lines.join('\n'), 'utf8');
	} catch (error) {
		console.error(`Failed to update Cargo.toml at ${filePath}:`, error);
		process.exit(1);
	}
}

updateJson(packageJsonPath);
updateJson(tauriConfigPath);
updateCargoToml(cargoTomlPath);

console.log(`\nSuccessfully synced version ${newVersion} across all files.`);
