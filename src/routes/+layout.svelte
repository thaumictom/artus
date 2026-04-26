<script lang="ts">
	import { ModeWatcher } from 'mode-watcher';
	import 'overlayscrollbars/overlayscrollbars.css';
	import '../app.css';
	import type { Snippet } from 'svelte';

	let { children }: { children?: Snippet } = $props();
</script>

<ModeWatcher defaultTheme="dark" />
{@render children?.()}
