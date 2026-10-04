<script>
	const commitHash = import.meta.env.VITE_ARTUS_COMMIT_HASH || 'unknown';
	const appVersion = import.meta.env.VITE_ARTUS_VERSION || 'unknown';
</script>

<div class="flex justify-between gap-2">
	<div class="flex-1">
		<p class="text-base">Build Commit</p>
		<p class="font-mono text-muted-foreground text-sm break-all">{commitHash}</p>
	</div>
	<div class="flex-1">
		<p class="text-base">App version</p>
		<p class="font-mono text-muted-foreground text-sm break-all">{appVersion}</p>
	</div>
</div>
