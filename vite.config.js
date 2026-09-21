import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
// @ts-expect-error node builtins are available in Vite config runtime
import { execSync } from "node:child_process";
import tailwindcss from '@tailwindcss/vite';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;
const COMMIT_HASH_LENGTH = 7;

/** @returns {import('vite').Plugin} */
function bundleWorldstateDataJson() {
	return {
		name: 'bundle-worldstate-data-json',
		enforce: 'pre',
		transform(code, id) {
			const normalizedId = id.replaceAll('\\', '/');
			if (
				!normalizedId.includes('/warframe-worldstate-data/dist/safeImport-') ||
				!code.includes('//#region safeImport.ts')
			) {
				return;
			}

			// The package uses import(path) for JSON, which Vite cannot discover and
			// leaves as broken runtime requests in the static Tauri production build.
			// Artus currently requests English only, so locale subdirectories can keep
			// using the package's built-in English fallback instead of bloating the app.
			const bundledLoader = code
				.replace(
					'//#region safeImport.ts',
					'//#region safeImport.ts\nconst bundledJsonModules = import.meta.glob("./data/*.json", { eager: true });',
				)
				.replace(
					'await import(path, { with: { type: "json" } })',
					'bundledJsonModules[path]',
				);

			if (bundledLoader === code) {
				this.error('Could not replace the warframe-worldstate-data JSON loader.');
			}

			return bundledLoader;
		},
	};
}

function resolveCommitHash() {
	// @ts-expect-error process is a nodejs global
	const ciHash = process.env.GITHUB_SHA;
	if (typeof ciHash === "string" && ciHash.trim().length > 0) {
		return ciHash.slice(0, COMMIT_HASH_LENGTH);
	}

	try {
		return execSync(`git rev-parse --short=${COMMIT_HASH_LENGTH} HEAD`, {
			stdio: ["ignore", "pipe", "ignore"],
		})
			.toString()
			.trim();
	} catch {
		return "unknown";
	}
}

function resolveAppVersion() {
	// @ts-expect-error process is a nodejs global
	const packageVersion = process.env.npm_package_version;
	if (typeof packageVersion === "string" && packageVersion.trim().length > 0) {
		return packageVersion.trim();
	}

	return "unknown";
}

const commitHash = resolveCommitHash();
const appVersion = resolveAppVersion();

// https://vite.dev/config/
export default defineConfig(async () => ({
	plugins: [
		bundleWorldstateDataJson(),
		nodePolyfills({ include: ['crypto', 'stream', 'vm'] }),
		tailwindcss(),
		sveltekit(),
	],
	optimizeDeps: {
		exclude: ['warframe-worldstate-parser', 'warframe-worldstate-data'],
		include: ['warframe-worldstate-parser > class-transformer', 'warframe-worldstate-parser > class-validator'],
	},
	build: { target: 'es2022' },
	define: {
		"import.meta.env.VITE_ARTUS_COMMIT_HASH": JSON.stringify(commitHash),
		"import.meta.env.VITE_ARTUS_VERSION": JSON.stringify(appVersion),
	},

	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	//
	// 1. prevent Vite from obscuring rust errors
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
				protocol: "ws",
				host,
				port: 1421,
			}
			: undefined,
		watch: {
			// 3. tell Vite to ignore watching `src-tauri`
			ignored: ["**/src-tauri/**"],
		},
	},
}));
