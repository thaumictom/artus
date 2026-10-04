<script lang="ts">
	import { Tooltip } from 'bits-ui';

	let {
		delayDuration = 200,
		skipDelayDuration = 100,
		...props
	}: Tooltip.ProviderProps = $props();
</script>

<Tooltip.Provider {delayDuration} {skipDelayDuration} {...props} />
