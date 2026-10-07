<script lang="ts">
	import AcknowledgementsDialog from '../components/AcknowledgementsDialog.svelte';

	let { onVersionTripleClick }: { onVersionTripleClick?: () => void } = $props();

	const commitHash = import.meta.env.VITE_ARTUS_COMMIT_HASH || 'unknown';
	const appVersion = import.meta.env.VITE_ARTUS_VERSION || 'unknown';
</script>

<div class="grid grid-cols-3 gap-2">
	<div class="min-w-0">
		<p class="text-base">Build Commit</p>
		<p class="font-mono text-muted-foreground text-sm break-all">{commitHash}</p>
	</div>
	<div class="min-w-0">
		<p class="text-base">App version</p>
		<button
			type="button"
			onclick={(event) => {
				if (event.detail === 3) onVersionTripleClick?.();
			}}
			class="p-0 font-mono text-muted-foreground text-sm text-left break-all cursor-pointer"
		>
			{appVersion}
		</button>
	</div>
	<div class="flex items-end">
		<AcknowledgementsDialog />
	</div>
</div>
