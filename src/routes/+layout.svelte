<script lang="ts">
	import { ModeWatcher } from 'mode-watcher';
	import 'overlayscrollbars/overlayscrollbars.css';
	import '../app.css';

	let { children } = $props();
</script>

<ModeWatcher defaultTheme="dark" />
{@render children?.()}
