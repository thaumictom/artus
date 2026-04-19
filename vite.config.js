import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
// @ts-expect-error node builtins are available in Vite config runtime
import { execSync } from "node:child_process";
import tailwindcss from '@tailwindcss/vite';

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;
const COMMIT_HASH_LENGTH = 7;

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

const commitHash = resolveCommitHash();

// https://vite.dev/config/
export default defineConfig(async () => ({
	plugins: [tailwindcss(), sveltekit()],
	define: {
		"import.meta.env.VITE_ARTUS_COMMIT_HASH": JSON.stringify(commitHash),
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
