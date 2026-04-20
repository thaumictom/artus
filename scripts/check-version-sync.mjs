import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

function readJson(filePath) {
	try {
		return JSON.parse(readFileSync(filePath, 'utf8'));
	} catch (error) {
		console.error(`Failed to read JSON from ${filePath}:`, error);
		process.exit(1);
	}
}

function readCargoPackageVersion(filePath) {
	let content;

	try {
		content = readFileSync(filePath, 'utf8');
	} catch (error) {
		console.error(`Failed to read Cargo.toml from ${filePath}:`, error);
		process.exit(1);
	}

	const lines = content.split(/\r?\n/);
	let inPackageSection = false;

	for (const rawLine of lines) {
		const line = rawLine.trim();

		if (line.startsWith('[') && line.endsWith(']')) {
			inPackageSection = line === '[package]';
			continue;
		}

		if (!inPackageSection) {
			continue;
		}

		const uncommentedLine = line.split('#', 1)[0].trim();
		const versionMatch = uncommentedLine.match(/^version\s*=\s*["']([^"']+)["']\s*$/);
		if (versionMatch) {
			return versionMatch[1];
		}
	}

	return null;
}

const rootDir = process.cwd();
const packageJsonPath = resolve(rootDir, 'package.json');
const tauriConfigPath = resolve(rootDir, 'src-tauri', 'tauri.conf.json');
const cargoTomlPath = resolve(rootDir, 'src-tauri', 'Cargo.toml');

const packageJson = readJson(packageJsonPath);
const tauriConfig = readJson(tauriConfigPath);
const packageVersion = packageJson.version;
const tauriVersion = tauriConfig.version;
const cargoVersion = readCargoPackageVersion(cargoTomlPath);

if (
	typeof packageVersion !== 'string' ||
	typeof tauriVersion !== 'string' ||
	typeof cargoVersion !== 'string'
) {
	console.error('Unable to compare versions: one of the version fields is missing or invalid.');
	process.exit(1);
}

if (packageVersion !== tauriVersion || packageVersion !== cargoVersion) {
	console.error('Version mismatch detected.');
	console.error(`package.json version: ${packageVersion}`);
	console.error(`src-tauri/tauri.conf.json version: ${tauriVersion}`);
	console.error(`src-tauri/Cargo.toml version: ${cargoVersion}`);
	console.error('Please keep all versions in sync before committing.');
	process.exit(1);
}

console.log(`Version check passed: ${packageVersion}`);
