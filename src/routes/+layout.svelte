<script lang="ts">
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from 'svelte-sonner';
	import 'overlayscrollbars/overlayscrollbars.css';
	import '../app.css';
	import type { Snippet } from 'svelte';

	let { children }: { children?: Snippet } = $props();
</script>

<Toaster position="bottom-center" theme="dark" />
<ModeWatcher defaultTheme="dark" />
{@render children?.()}
