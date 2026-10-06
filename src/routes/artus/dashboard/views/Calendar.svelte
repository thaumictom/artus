<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import type { DayEvent } from 'warframe-worldstate-parser';
	import Button from '$lib/components/Button.svelte';
	import WorldStateMissionCard from '$lib/components/WorldStateMissionCard.svelte';
	import ViewToolbar from '$lib/components/ViewToolbar.svelte';
	import WarframeItem from '$lib/components/WarframeItem.svelte';
	import {
		calendarCompletions, getCalendarDays, hasCalendarTodo, initializeCalendarCompletions,
		isCalendarDayCompleted, setCalendarDaysCompleted,
	} from '$lib/calendar-completions.svelte';
	import type { DashboardViewProps } from './view-types';

	const DATE_FORMATTER = new Intl.DateTimeFormat(undefined, {
		weekday: 'long',
		month: 'long',
		day: 'numeric',
		year: 'numeric',
		timeZone: 'UTC',
	});

	let { world, now }: DashboardViewProps = $props();
	let calendar = $derived(world.calendar);
	let days = $derived(getCalendarDays(calendar));
	let todosOnly = $state(false);
	let visibleDays = $derived(todosOnly
		? days.map((day) => ({ ...day, events: day.events.filter((event) => hasCalendarTodo([event])) }))
			.filter((day) => day.events.length > 0)
		: days);
	let uncheckedIds = $derived(days.filter((day) => hasCalendarTodo(day.events)
		&& !isCalendarDayCompleted(day.id)).map((day) => day.id));
	let saving = $state(false);
	let error = $state('');

	onMount(() => {
		void initializeCalendarCompletions().catch((cause) => {
			console.error('Could not load calendar completions:', cause);
			error = 'Could not load completed To Dos. Reopen the calendar to try again.';
		});
	});

	async function toggleCompleted(ids: string[], completed: boolean) {
		if (!calendarCompletions.loaded || saving || ids.length === 0) return;
		saving = true;
		error = '';
		try {
			await setCalendarDaysCompleted(ids, completed);
		} catch (cause) {
			console.error('Could not save calendar completion:', cause);
			error = 'Could not save completed To Dos. Please try again.';
		} finally {
			saving = false;
		}
	}

	function eventTitle(event: DayEvent) {
		return (
			event.challenge?.title ||
			event.upgrade?.title ||
			event.reward ||
			event.dialogueName ||
			event.type
		);
	}

	function eventDescription(event: DayEvent) {
		return event.challenge?.description || event.upgrade?.description || event.dialogueConvo;
	}

	function eventGroups(events: DayEvent[]): DayEvent[][] {
		const groups = new Map<string, DayEvent[]>();
		for (const event of events) {
			const key = event.type.toLowerCase();
			const group = groups.get(key);
			if (group) group.push(event);
			else groups.set(key, [event]);
		}
		return [...groups.values()];
	}

	function eventAppearance(type: string) {
		switch (type.toLowerCase()) {
			case 'to do':
				return { icon: 'material-symbols:checklist-rounded', color: 'text-[#88c0d0]' };
			case 'override':
				return { icon: 'material-symbols:tune-rounded', color: 'text-[#b48ead]' };
			case 'big prize!':
				return {
					icon: 'material-symbols:featured-seasonal-and-gifts-rounded',
					color: 'text-[#ebcb8b]',
				};
			case 'birthday':
				return { icon: 'material-symbols:cake-rounded', color: 'text-[#d08770]' };
			default:
				return { icon: 'material-symbols:event-rounded', color: 'text-muted-foreground' };
		}
	}
</script>

<section aria-label="1999 Calendar" class="flex flex-col gap-4 min-w-0">
	<ViewToolbar>
		{#snippet summary()}
			{#if calendar}
				<span class="pr-4">{calendar.season}</span>
				<span class="pl-4 text-muted-foreground">Loop {calendar.yearIteration}</span>
			{/if}
		{/snippet}
		{#snippet actions()}
			<Button
				variant={todosOnly ? 'surface' : 'default'}
				class="inline-flex items-center gap-1.5 text-base"
				aria-pressed={todosOnly}
				onclick={() => (todosOnly = !todosOnly)}
			>
				<Icon icon="lucide:list-filter" class="size-4" /> To Dos only
			</Button>
			<Button
				variant="primary"
				class="inline-flex items-center gap-1.5 text-base"
				disabled={!calendarCompletions.loaded || saving || uncheckedIds.length === 0}
				onclick={() => toggleCompleted(uncheckedIds, true)}
			>
				<Icon icon="lucide:check" class="size-4" /> Complete all
			</Button>
		{/snippet}
	</ViewToolbar>
	{#if error}<p role="alert" class="text-danger text-sm">{error}</p>{/if}
	<ul class="flex flex-col gap-3">
		{#each visibleDays as { id, date, events } (id)}
			{@const hasTodo = hasCalendarTodo(events)}
			<WorldStateMissionCard
				node={DATE_FORMATTER.format(date)}
				{now}
				completed={hasTodo && isCalendarDayCompleted(id)}
				disabled={hasTodo && (!calendarCompletions.loaded || saving)}
				onCompletedChange={hasTodo ? (checked) => void toggleCompleted([id], checked) : undefined}
			>
				<div class="divide-y divide-surface">
					{#each eventGroups(events) as group}
						{@const type = group[0].type}
						{@const appearance = eventAppearance(type)}
						{@const sideBySide = ['big prize!', 'override'].includes(type.toLowerCase())}
						<div class="py-4 first:pt-0 last:pb-0">
							<div class="flex items-center gap-2 mb-3">
								<Icon icon={appearance.icon} class={`size-4 shrink-0 ${appearance.color}`} />
								<p class="font-medium text-muted-foreground text-xs uppercase tracking-widest">{type}</p>
							</div>
							<div class="grid gap-4 grid-cols-1 {sideBySide && group.length > 1
								? group.length === 2 ? 'sm:grid-cols-2' : 'sm:grid-cols-3'
								: ''}">
								{#each group as event, index}
									<div class="min-w-0 {index > 0
										? sideBySide
											? 'border-t border-border-secondary pt-4 sm:border-t-0 sm:border-l sm:pt-0 sm:pl-4'
											: 'border-t border-surface pt-4'
										: ''}">
										{#if event.uniqueName}
											<WarframeItem item={event.uniqueName} name={eventTitle(event)} />
										{:else}
											<h3 class="font-semibold break-words">{eventTitle(event)}</h3>
										{/if}
										{#if eventDescription(event)}
											<p class="mt-2 text-muted-foreground text-sm whitespace-pre-line break-words">{eventDescription(event)}</p>
										{/if}
									</div>
								{/each}
							</div>
						</div>
					{/each}
				</div>
			</WorldStateMissionCard>
		{:else}
			<li class="p-6 border border-surface text-muted-foreground text-sm text-center">
				{todosOnly ? 'No To Dos in this snapshot.' : 'No calendar events in this snapshot.'}
			</li>
		{/each}
	</ul>
</section>
