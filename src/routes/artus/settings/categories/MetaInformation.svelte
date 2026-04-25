<script>
	const commitHash = import.meta.env.VITE_ARTUS_COMMIT_HASH || 'unknown';
	const appVersion = import.meta.env.VITE_ARTUS_VERSION || 'unknown';
</script>

<div class="flex justify-between gap-2">
	<div class="flex-1">
		<p class="text-sm">Build Commit</p>
		<p class="font-mono text-muted-foreground text-xs break-all">{commitHash}</p>
	</div>
	<div class="flex-1">
		<p class="text-sm">App version</p>
		<p class="font-mono text-muted-foreground text-xs break-all">{appVersion}</p>
	</div>
</div>
