<script lang="ts">
	import { onMount } from 'svelte';

	import { getWarframeLogPath, setWarframeLogPath } from '../settings-api';

	let warframeLogPathInput = $state('');
	let isLoadingWarframeLogPath = $state(true);
	let isSavingWarframeLogPath = $state(false);
	let warframeLogPathStatus = $state<string | null>(null);

	function sanitizeWarframeLogPathInput(input: string): string {
		let sanitized = input.trim();

		// Windows copy-path often wraps the full path in quotes.
		while (
			sanitized.length >= 2 &&
			((sanitized.startsWith('"') && sanitized.endsWith('"')) ||
				(sanitized.startsWith("'") && sanitized.endsWith("'")))
		) {
			sanitized = sanitized.slice(1, -1).trim();
		}

		return sanitized;
	}

	async function loadWarframeLogPath() {
		isLoadingWarframeLogPath = true;
		warframeLogPathStatus = null;

		try {
			warframeLogPathInput = await getWarframeLogPath();
		} catch (error) {
			warframeLogPathStatus = String(error);
		} finally {
			isLoadingWarframeLogPath = false;
		}
	}

	async function saveWarframeLogPath() {
		if (isLoadingWarframeLogPath || isSavingWarframeLogPath) {
			return;
		}

		const sanitizedPath = sanitizeWarframeLogPathInput(warframeLogPathInput);
		warframeLogPathInput = sanitizedPath;

		isSavingWarframeLogPath = true;
		warframeLogPathStatus = null;

		try {
			const savedPath = await setWarframeLogPath(sanitizedPath);
			warframeLogPathInput = savedPath;
			warframeLogPathStatus = 'Saved';
		} catch (error) {
			warframeLogPathStatus = String(error);
		} finally {
			isSavingWarframeLogPath = false;
		}
	}

	onMount(() => {
		void loadWarframeLogPath();
	});
</script>

<div class="flex flex-col gap-2">
	<div>
		<p>Warframe log path</p>
		<p class="text-muted-foreground text-xs">Set this to your Warframe EE.log file path</p>
	</div>

	<div class="flex sm:flex-row flex-col gap-2">
		<input
			type="text"
			bind:value={warframeLogPathInput}
			class="px-2 py-1 border rounded w-full text-sm"
			placeholder="C:\\Users\\You\\AppData\\Local\\Warframe\\EE.log"
			disabled={isLoadingWarframeLogPath || isSavingWarframeLogPath}
		/>
		<button
			type="button"
			class="hover:bg-muted px-3 py-1 border rounded text-sm"
			onclick={() => void saveWarframeLogPath()}
			disabled={isLoadingWarframeLogPath || isSavingWarframeLogPath}
		>
			{isSavingWarframeLogPath ? 'Saving...' : 'Save'}
		</button>
	</div>

	{#if warframeLogPathStatus}
		<p class="text-muted-foreground text-xs">{warframeLogPathStatus}</p>
	{/if}
</div>
