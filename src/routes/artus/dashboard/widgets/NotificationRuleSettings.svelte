<script lang="ts">
	import Icon from '@iconify/svelte';
	import { mode } from 'mode-watcher';
	import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
	import type { WorldState } from 'warframe-worldstate-parser';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Switch from '$lib/components/Switch.svelte';
	import { requestDesktopNotificationPermission } from '$lib/notifications.svelte';
	import {
		config,
		updateSetting,
		type FissureNotificationCategory,
		type NotificationRules,
	} from '$lib/settings.svelte';

	let {
		world,
		triggerClass,
		showTrigger = true,
		open = $bindable(false),
	}: {
		world: WorldState | null;
		triggerClass?: string;
		showTrigger?: boolean;
		open?: boolean;
	} = $props();
	let saveError = $state<string | null>(null);
	let saveQueue = Promise.resolve();
	let scrollbarTheme = $derived(mode.current === 'light' ? 'os-theme-dark' : 'os-theme-light');
	let fissures = $derived(world?.fissures ?? []);

	const standardEras = ['Lith', 'Meso', 'Neo', 'Axi', 'Requiem', 'Omnia'];
	const standardMissionTypes = [
		'Alchemy',
		'Assassination',
		'Capture',
		'Defense',
		'Disruption',
		'Excavation',
		'Exterminate',
		'Hijack',
		'Interception',
		'Mobile Defense',
		'Rescue',
		'Sabotage',
		'Spy',
		'Survival',
		'Void Armageddon',
		'Void Cascade',
		'Void Flood',
	];
	const categories: { value: FissureNotificationCategory; label: string }[] = [
		{ value: 'normal', label: 'Normal' },
		{ value: 'steelPath', label: 'Steel Path' },
		{ value: 'voidStorm', label: 'Void Storm' },
	];
	const simpleRules = [
		{ key: 'alerts', title: 'Alerts', description: 'Notify when a new alert appears.' },
		{ key: 'invasions', title: 'Invasions', description: 'Notify when a new invasion begins.' },
		{ key: 'dailyDeals', title: 'Daily deals', description: 'Notify when Darvo offers a new deal.' },
		{ key: 'baro', title: "Baro Ki'Teer", description: 'Notify when Baro arrives at a relay.' },
	] as const;
	let eraOptions = $derived(
		[...new Set([...standardEras, ...fissures.map((fissure) => fissure.tier)])].sort(
			(a, b) => standardEras.indexOf(a) - standardEras.indexOf(b),
		),
	);
	let missionTypeOptions = $derived(
		[...new Set([...standardMissionTypes, ...fissures.map((fissure) => fissure.missionType)])].sort(),
	);

	function saveRules() {
		saveError = null;
		saveQueue = saveQueue
			.catch(() => undefined)
			.then(() => updateSetting('notification_rules'))
			.catch((error) => {
				console.error('Could not save notification rules:', error);
				saveError = 'Could not save notification rules.';
			});
	}

	function setRule(rule: Exclude<keyof NotificationRules, 'fissures'>, enabled: boolean) {
		config.notification_rules[rule] = enabled;
		saveRules();
		requestPermissionWhenEnabled(enabled);
	}

	function requestPermissionWhenEnabled(enabled: boolean) {
		if (enabled && config.desktop_notifications_enabled) {
			void requestDesktopNotificationPermission();
		}
	}

	function toggleFilter<T extends string>(key: 'eras' | 'missionTypes' | 'categories', value: T) {
		const values = config.notification_rules.fissures[key] as T[];
		(config.notification_rules.fissures[key] as T[]) = values.includes(value)
			? values.filter((item) => item !== value)
			: [...values, value];
		saveRules();
	}
</script>

{#snippet trigger()}
	{#if showTrigger}
		<Button
			size="icon"
			class={triggerClass}
			title="Configure notifications"
			aria-label="Configure notifications"
		>
			<Icon icon="material-symbols:notification-add-outline-rounded" class="size-4 shrink-0" />
		</Button>
	{/if}
{/snippet}

{#snippet title()}World state notifications{/snippet}

{#snippet description()}
	Choose the events Artus should watch. Active rules refresh the world state every five minutes.
{/snippet}

{#snippet dialogClose()}<Button>Done</Button>{/snippet}

<Dialog
	bind:open
	trigger={showTrigger ? trigger : undefined}
	{title}
	{description}
	{dialogClose}
>
	<OverlayScrollbarsComponent
		defer
		class="flex-1 mr-1.75 min-w-0 min-h-0"
		options={{ scrollbars: { theme: scrollbarTheme, autoHide: 'move' } }}
	>
		<div class="flex flex-col gap-5 pr-4.25 pl-6">
			<section class="border">
				<div class="flex justify-between items-center gap-4 p-4">
					<div>
						<h3 class="font-medium text-sm">Fissure missions</h3>
						<p class="text-muted-foreground text-xs">Notify only when all selected filters match.</p>
					</div>
					<Switch
						checked={config.notification_rules.fissures.enabled}
						onCheckedChange={(enabled) => {
							config.notification_rules.fissures.enabled = enabled;
							saveRules();
							requestPermissionWhenEnabled(enabled);
						}}
						aria-label="Fissure mission notifications"
					/>
				</div>
				<div class:opacity-50={!config.notification_rules.fissures.enabled} class="flex flex-col gap-4 px-4 pb-4">
					<div>
						<div class="flex justify-between mb-1.5 text-xs">
							<span class="font-semibold text-muted-foreground">Mission category</span>
							<span class="text-muted-foreground">None selected means any</span>
						</div>
						<div class="flex flex-wrap gap-1.5">
							{#each categories as category (category.value)}
								<button
									type="button"
									disabled={!config.notification_rules.fissures.enabled}
									aria-pressed={config.notification_rules.fissures.categories.includes(category.value)}
									onclick={() => toggleFilter('categories', category.value)}
									class="aria-pressed:bg-accent aria-pressed:border-accent px-2.5 py-1 border aria-pressed:text-accent-foreground text-xs cursor-pointer disabled:cursor-not-allowed"
								>
									{category.label}
								</button>
							{/each}
						</div>
					</div>
					<div>
						<div class="flex justify-between mb-1.5 text-xs">
							<span class="font-semibold text-muted-foreground">Era</span>
							<span class="text-muted-foreground">None selected means any</span>
						</div>
						<div class="flex flex-wrap gap-1.5">
							{#each eraOptions as era (era)}
								<button
									type="button"
									disabled={!config.notification_rules.fissures.enabled}
									aria-pressed={config.notification_rules.fissures.eras.includes(era)}
									onclick={() => toggleFilter('eras', era)}
									class="aria-pressed:bg-accent aria-pressed:border-accent px-2.5 py-1 border aria-pressed:text-accent-foreground text-xs cursor-pointer disabled:cursor-not-allowed"
								>
									{era}
								</button>
							{/each}
						</div>
					</div>
					<div>
						<div class="flex justify-between mb-1.5 text-xs">
							<span class="font-semibold text-muted-foreground">Mission type</span>
							<span class="text-muted-foreground">None selected means any</span>
						</div>
						<div class="flex flex-wrap gap-1.5">
							{#each missionTypeOptions as missionType (missionType)}
								<button
									type="button"
									disabled={!config.notification_rules.fissures.enabled}
									aria-pressed={config.notification_rules.fissures.missionTypes.includes(missionType)}
									onclick={() => toggleFilter('missionTypes', missionType)}
									class="aria-pressed:bg-accent aria-pressed:border-accent px-2.5 py-1 border aria-pressed:text-accent-foreground text-xs cursor-pointer disabled:cursor-not-allowed"
								>
									{missionType}
								</button>
							{/each}
						</div>
					</div>
				</div>
			</section>

			{#each simpleRules as rule (rule.key)}
				<section class="flex justify-between items-center gap-4 p-4 border">
					<div>
						<h3 class="font-medium text-sm">{rule.title}</h3>
						<p class="text-muted-foreground text-xs">{rule.description}</p>
					</div>
					<Switch
						checked={config.notification_rules[rule.key]}
						onCheckedChange={(enabled) => setRule(rule.key, enabled)}
						aria-label={`${rule.title} notifications`}
					/>
				</section>
			{/each}
		</div>
	</OverlayScrollbarsComponent>
	{#if saveError}<p role="alert" class="px-6 text-danger text-sm">{saveError}</p>{/if}
</Dialog>
