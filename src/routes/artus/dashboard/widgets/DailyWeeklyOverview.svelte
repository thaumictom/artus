<script lang="ts">
	import Icon from '@iconify/svelte';
	import Button from '$lib/components/Button.svelte';
	import ViewCard from '$lib/components/ViewCard.svelte';
	import BountyRotationRewards from '$lib/components/BountyRotationRewards.svelte';
	import type { BountyRotation, BountyRotationKind } from '$lib/bounty-rotation-rewards';
	import { oracleBounties } from '$lib/oracle-bounties.svelte';
	import { calendarCompletions, getCalendarDays, hasCalendarTodo, isCalendarDayCompleted } from '$lib/calendar-completions.svelte';
	import { clanInitiativeCompletions, getClanInitiativeRewards, isClanInitiativeRewardCompleted } from '$lib/clan-initiative-completions.svelte';
	import type { DashboardView } from '../dashboard-views';
	import { isCurrent, validDate, type DashboardViewProps } from '../views/view-types';

	let { world, now, localNow, onSelect }: DashboardViewProps = $props();
	type Overview = { view: DashboardView; title: string; summary: string; details?: string; expiry?: Date; completed?: boolean; clock?: number; rotations?: { kind: BountyRotationKind; rotation: BountyRotation }[] };
	const syndicateNames = ['Steel Meridian', 'Red Veil', 'Cephalon Suda', 'Perrin Sequence', 'New Loka', 'Arbiters of Hexis'];
	const bountyNames = ['Solaris United', 'Ostrons', 'Entrati'];
	const count = (value: number, singular: string, plural = `${singular}s`) => `${value} ${value === 1 ? singular : plural}`;
	function earliest(dates: (Date | undefined)[]) {
		return dates.filter(validDate).toSorted((a, b) => a.getTime() - b.getTime())[0];
	}

	let cards: Overview[] = $derived.by(() => {
		const sortie = isCurrent(world.sortie, now) ? world.sortie : undefined;
		const archon = isCurrent(world.archonHunt, now) ? world.archonHunt : undefined;
		const syndicates = (world.syndicateMissions ?? []).filter((item) => isCurrent(item, now) && syndicateNames.includes(item.syndicateKey));
		const hubBounties = (world.syndicateMissions ?? []).filter((item) => isCurrent(item, now) && bountyNames.includes(item.syndicateKey))
			.flatMap((item) => (item.jobs ?? []).filter((job) => isCurrent(job, now)));
		const oracle = oracleBounties.snapshot;
		const oracleCurrent = oracle?.expiry != null && oracle.expiry > localNow;
		const extraBounties = oracleCurrent ? Object.values(oracle.bounties).reduce((total, entries) => total + entries.length, 0) : 0;
		const field = oracle?.fieldBounties && oracle.fieldBounties.expiry > localNow ? oracle.fieldBounties : undefined;
		const fieldLocations = field ? Object.keys(field.CetusSyndicate).length + Object.keys(field.SolarisSyndicate).length + Object.keys(field.EntratiSyndicate).length : 0;
		const archimedeas = (world.archimedeas ?? []).filter((item) => isCurrent(item, now));
		const descendia = isCurrent(world.descendia, now) ? world.descendia : undefined;
		const initiative = isCurrent(world.clanWeeklyInitiative, now) ? world.clanWeeklyInitiative : undefined;
		const rewards = getClanInitiativeRewards(initiative);
		const remainingRewards = rewards.filter(({ id }) => !isClanInitiativeRewardCompleted(id)).length;
		const calendar = isCurrent(world.calendar, now) ? world.calendar : undefined;
		const todos = (calendar ? getCalendarDays(calendar) : []).filter((day) => hasCalendarTodo(day.events));
		const remainingTodos = todos.filter((day) => !isCalendarDayCompleted(day.id)).length;
		const circuit = world.duviriCycle?.choices ?? [];
		const nightwave = (world.nightwave?.activeChallenges ?? []).filter((item) => isCurrent(item, now));
		return [
			{
				view: 'Sortie', title: 'Sortie', summary: sortie?.boss || 'No current Sortie',
				details: sortie?.variants.map((mission) => mission.missionType).filter(Boolean).join(' · '), expiry: sortie?.expiry,
			},
			{
				view: 'ArchonHunt', title: 'Archon Hunt', summary: archon?.boss || 'No current Archon Hunt',
				details: archon?.missions.map((mission) => mission.type).filter(Boolean).join(' · '), expiry: archon?.expiry,
			},
			{
				view: 'SyndicateMissions', title: 'Syndicates',
				summary: syndicates.length ? count(syndicates.reduce((total, item) => total + (item.nodes?.length ?? 0), 0), 'mission') : 'No current syndicate missions',
				details: syndicates.length ? `Across ${count(syndicates.length, 'syndicate')}` : undefined,
				expiry: earliest(syndicates.map((item) => item.expiry)),
			},
			{
				view: 'Bounties', title: 'Bounties',
				summary: hubBounties.length + extraBounties ? `${hubBounties.length + extraBounties} hub & challenge bounties` : 'No current hub or challenge bounties',
				details: [
					fieldLocations ? `${count(fieldLocations, 'field location')} with additional bounties` : 'Open-world, Zariman, Cavia and Hex',
				].filter(Boolean).join(' · '),
				rotations: oracleCurrent ? [
					...(oracle.rot ? [{ kind: 'cetus' as const, rotation: oracle.rot }] : []),
					...(oracle.vaultRot ? [{ kind: 'vault' as const, rotation: oracle.vaultRot }] : []),
				] : [],
				clock: localNow,
				expiry: earliest([...hubBounties.map((job) => job.expiry), ...(oracleCurrent ? [new Date(oracle.expiry!)] : []), ...(field ? [new Date(field.expiry)] : [])]),
			},
			{
				view: 'Circuit', title: 'The Circuit', summary: circuit.length ? 'Weekly reward choices' : 'No current Circuit choices',
				details: circuit.map((choice) => `${choice.category === 'hard' ? 'Steel Path' : 'Normal'}: ${count(choice.choices.length, 'choice')}`).join(' · '),
			},
			{
				view: 'Archimedea', title: 'Archimedea', summary: archimedeas.length ? count(archimedeas.reduce((total, item) => total + item.missions.length, 0), 'mission') : 'No current Archimedea',
				details: 'Deep and Temporal · weekly modifiers', expiry: earliest(archimedeas.map((item) => item.expiry)),
			},
			{
				view: 'Descendia', title: 'Descendia', summary: descendia ? count(descendia.challenges.length, 'floor') : 'No current Descendia',
				details: 'Weekly objectives, enemies and penances', expiry: descendia?.expiry,
			},
			{
				view: 'ClanInitiative', title: 'Clan Initiative',
				summary: !rewards.length ? 'No current Clan Initiative' : clanInitiativeCompletions.loaded ? `${count(remainingRewards, 'reward')} remaining` : 'Loading completion status…',
				details: initiative?.bonusRegion ? `Bonus region: ${initiative.bonusRegion}` : undefined,
				expiry: initiative?.expiry,
				completed: clanInitiativeCompletions.loaded && rewards.length > 0 && remainingRewards === 0,
			},
			{
				view: 'Calendar', title: '1999 Calendar',
				summary: !calendar ? 'No current calendar' : calendarCompletions.loaded ? `${count(remainingTodos, 'to-do day')} remaining` : 'Loading completion status…',
				details: calendar ? `${calendar.season} · Loop ${calendar.yearIteration}` : undefined, expiry: calendar?.expiry,
				completed: calendarCompletions.loaded && todos.length > 0 && remainingTodos === 0,
			},
			{
				view: 'Nightwave', title: 'Nightwave', summary: nightwave.length ? count(nightwave.length, 'challenge') : 'No current Nightwave challenges',
				details: nightwave.length ? `${count(nightwave.filter((item) => item.isDaily).length, 'daily', 'daily')} · ${count(nightwave.filter((item) => !item.isDaily && !item.isElite).length, 'weekly', 'weekly')} · ${count(nightwave.filter((item) => !item.isDaily && item.isElite).length, 'elite weekly', 'elite weekly')}` : undefined,
				expiry: earliest(nightwave.map((item) => item.expiry)),
			},
		];
	});
</script>

<section class="flex flex-col gap-3 min-w-0" aria-labelledby="daily-weekly-heading">
	<h2 id="daily-weekly-heading" class="font-semibold">Daily &amp; Weekly</h2>
	<ul class="grid grid-cols-1 gap-3 sm:grid-cols-2">
		{#each cards as card (card.view)}
			<ViewCard now={card.clock ?? now} expiry={card.expiry} completed={card.completed} onActivate={() => onSelect(card.view)}>
				<h3>
					<Button variant="link" size="none" class="flex w-full items-center justify-between gap-3 text-left text-foreground font-semibold" onclick={() => onSelect(card.view)}>
						{card.title}<Icon icon="lucide:arrow-up-right" class="size-4 shrink-0 text-muted-foreground" />
					</Button>
				</h3>
				<p class="mt-2 text-sm {card.completed ? 'text-accent' : 'text-foreground'}">
					{#if card.completed}<Icon icon="lucide:check" class="mr-1 inline size-4" />{/if}{card.summary}
				</p>
				{#if card.details}<p class="mt-1 text-sm text-muted-foreground">{card.details}</p>{/if}
				{#if card.rotations?.length}
					<div class="mt-3 flex flex-col gap-3 border-t border-border-secondary pt-3">
						{#each card.rotations as rotation (rotation.kind)}
							<BountyRotationRewards {...rotation} />
						{/each}
					</div>
				{/if}
			</ViewCard>
		{/each}
	</ul>
</section>
