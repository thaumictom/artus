<script lang="ts">
	import { ModeWatcher } from 'mode-watcher';

	let { children } = $props();
	import '../app.css';
</script>

<ModeWatcher defaultTheme="dark" />
{@render children?.()}
