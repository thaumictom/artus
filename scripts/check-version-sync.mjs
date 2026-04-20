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

const rootDir = process.cwd();
const packageJsonPath = resolve(rootDir, 'package.json');
const tauriConfigPath = resolve(rootDir, 'src-tauri', 'tauri.conf.json');

const packageJson = readJson(packageJsonPath);
const tauriConfig = readJson(tauriConfigPath);
const packageVersion = packageJson.version;
const tauriVersion = tauriConfig.version;

if (typeof packageVersion !== 'string' || typeof tauriVersion !== 'string') {
	console.error('Unable to compare versions: one of the version fields is missing or invalid.');
	process.exit(1);
}

if (packageVersion !== tauriVersion) {
	console.error('Version mismatch detected.');
	console.error(`package.json version: ${packageVersion}`);
	console.error(`src-tauri/tauri.conf.json version: ${tauriVersion}`);
	console.error('Please keep both versions in sync before committing.');
	process.exit(1);
}

console.log(`Version check passed: ${packageVersion}`);
