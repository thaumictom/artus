<script lang="ts">
	import BaroKiTeer from '../widgets/BaroKiTeer.svelte';
	import WeaponResets from '../widgets/WeaponResets.svelte';
	import DailyWeeklyOverview from '../widgets/DailyWeeklyOverview.svelte';
	import type { DashboardViewProps } from './view-types';

	let { world, now, localNow, onSelect }: DashboardViewProps = $props();
</script>

<section class="flex flex-col gap-6 min-w-0" aria-label="Home">
	<div class="flex flex-col gap-2">
		<BaroKiTeer trader={world.voidTrader} {now} onOpenInventory={() => onSelect('BaroInventory')} />
		<!-- Weapon rotations follow UTC wall time independently of the world-state timestamp. -->
		<WeaponResets now={localNow}
			onOpen={(source) => onSelect(source === 'tenet' ? 'TenetWeapons' : 'CodaWeapons')} />
	</div>
	<DailyWeeklyOverview {world} {now} {localNow} {onSelect} />
</section>
