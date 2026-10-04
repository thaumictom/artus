<script lang="ts">
	import CycleWidgets from '../widgets/CycleWidgets.svelte';
	import WeaponResets from '../widgets/WeaponResets.svelte';
	import type { DashboardViewProps } from './view-types';

	let { world, now, localNow, onSelect }: DashboardViewProps = $props();
</script>

<div class="flex flex-col gap-2">
	<CycleWidgets {world} {now} onOpenBaro={() => onSelect('BaroInventory')} />
	<!-- Fixed UTC rotations use wall time independently of the world-state timestamp. -->
	<WeaponResets
		now={localNow}
		onOpen={(source) => onSelect(source === 'tenet' ? 'TenetWeapons' : 'CodaWeapons')}
	/>
</div>
