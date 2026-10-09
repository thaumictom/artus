<script lang="ts">
	import BaroKiTeer from '../widgets/BaroKiTeer.svelte';
	import CycleWidgets from '../widgets/CycleWidgets.svelte';
	import WeaponResets from '../widgets/WeaponResets.svelte';
	import DailyWeeklyOverview from '../widgets/DailyWeeklyOverview.svelte';
	import type { DashboardViewProps } from './view-types';

	let { world, now, localNow, onSelect }: DashboardViewProps = $props();
</script>

<section class="@container/home flex flex-col gap-6 min-w-0" aria-label="Home">
	<div class="flex flex-col gap-2">
		<section
			aria-label="World cycles and Baro Ki'Teer"
			class="gap-2 grid grid-cols-1 @[30rem]/home:grid-cols-2 @[40rem]/home:grid-cols-3 @[80rem]/home:grid-cols-6"
		>
			<CycleWidgets {world} {now} />
			<BaroKiTeer
				trader={world.voidTrader}
				{now}
				onOpenInventory={() => onSelect('BaroInventory')}
			/>
		</section>
		<!-- Weapon rotations follow UTC wall time independently of the world-state timestamp. -->
		<WeaponResets
			now={localNow}
			onOpen={(source) => onSelect(source === 'tenet' ? 'TenetWeapons' : 'CodaWeapons')}
		/>
	</div>
	<DailyWeeklyOverview {world} {now} {localNow} {onSelect} />
</section>
