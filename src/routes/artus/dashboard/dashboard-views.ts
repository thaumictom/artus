import type { Component } from 'svelte';
import type { WorldState } from 'warframe-worldstate-parser';
import { alertCompletions, isAlertCompleted } from '$lib/alert-completions.svelte';
import { isBaroActive } from './baro';
import { isCurrent } from './views/view-types';
import type { DashboardViewProps } from './views/view-types';
import FissureMissions from './views/FissureMissions.svelte';
import News from './views/News.svelte';
import Alerts from './views/Alerts.svelte';
import Invasions from './views/Invasions.svelte';
import Sortie from './views/Sortie.svelte';
import ArchonHunt from './views/ArchonHunt.svelte';
import Nightwave from './views/Nightwave.svelte';
import SyndicateMissions from './views/SyndicateMissions.svelte';
import Arbitration from './views/Arbitration.svelte';
import KuvaMissions from './views/KuvaMissions.svelte';
import Events from './views/Events.svelte';
import DailyDeals from './views/DailyDeals.svelte';
import FlashSales from './views/FlashSales.svelte';
import GlobalUpgrades from './views/GlobalUpgrades.svelte';
import Circuit from './views/Circuit.svelte';
import Archimedea from './views/Archimedea.svelte';
import Descendia from './views/Descendia.svelte';
import Calendar from './views/Calendar.svelte';
import SentientOutposts from './views/SentientOutposts.svelte';
import Simaris from './views/Simaris.svelte';
import Conclave from './views/Conclave.svelte';
import PersistentEnemies from './views/PersistentEnemies.svelte';
import Construction from './views/Construction.svelte';
import WeeklyChallenges from './views/WeeklyChallenges.svelte';
import DarkSectors from './views/DarkSectors.svelte';
import Kinepage from './views/Kinepage.svelte';
import Faceoff from './views/Faceoff.svelte';
import BaroInventory from './views/BaroInventory.svelte';
import TenetWeapons from './views/TenetWeapons.svelte';
import CodaWeapons from './views/CodaWeapons.svelte';
import Acrithis from './views/Acrithis.svelte';
import SteelPath from './views/SteelPath.svelte';
import PrimeResurgence from './views/PrimeResurgence.svelte';
import ClanInitiative from './views/ClanInitiative.svelte';
import CommunityCampaign from './views/CommunityCampaign.svelte';
import WorldCycles from './views/WorldCycles.svelte';

export const dashboardViews = [
	{ value: 'WorldCycles', label: 'World cycles', component: WorldCycles },
	{ value: 'fissures', label: 'Fissure Missions', component: FissureMissions },
	{ value: 'news', label: 'News', component: News },
	{
		value: 'BaroInventory', label: "Baro Ki'Teer", component: BaroInventory,
		statusLabel: (world, now) => isBaroActive(world.voidTrader, now) ? 'Active' : undefined,
	},
	{ value: 'TenetWeapons', label: 'Tenet Weapons', component: TenetWeapons },
	{ value: 'CodaWeapons', label: 'Coda Weapons', component: CodaWeapons },
	{ value: 'Acrithis', label: 'Acrithis', component: Acrithis },
	{
		value: 'Alerts', label: "Alerts", component: Alerts,
		itemCount: (world, now) => alertCompletions.loaded
			? (world.alerts ?? []).filter((item) => isCurrent(item, now)
				&& !isAlertCompleted(item.id)).length
			: 0,
	},
	{ value: 'Invasions', label: "Invasions", component: Invasions },
	{ value: 'Sortie', label: "Sortie", component: Sortie },
	{ value: 'ArchonHunt', label: "Archon Hunt", component: ArchonHunt },
	{ value: 'Nightwave', label: "Nightwave", component: Nightwave },
	{ value: 'SyndicateMissions', label: "Syndicates & Bounties", component: SyndicateMissions },
	{ value: 'Arbitration', label: "Arbitration", component: Arbitration },
	{ value: 'KuvaMissions', label: "Kuva Missions", component: KuvaMissions },
	{ value: 'Events', label: "Events", component: Events },
	{ value: 'DailyDeals', label: "Daily Deals", component: DailyDeals },
	{ value: 'FlashSales', label: "Featured Market Items", component: FlashSales },
	{ value: 'GlobalUpgrades', label: "Global Boosters", component: GlobalUpgrades },
	{ value: 'Circuit', label: "The Circuit", component: Circuit },
	{ value: 'Archimedea', label: "Archimedea", component: Archimedea },
	{ value: 'Descendia', label: "Descendia", component: Descendia },
	{ value: 'Calendar', label: "1999 Calendar", component: Calendar },
	{ value: 'SentientOutposts', label: "Sentient Outposts", component: SentientOutposts },
	{ value: 'Simaris', label: "Simaris", component: Simaris },
	{ value: 'Conclave', label: "Conclave Challenges", component: Conclave },
	{ value: 'PersistentEnemies', label: "Persistent Enemies", component: PersistentEnemies },
	{ value: 'Construction', label: "Construction", component: Construction },
	{ value: 'WeeklyChallenges', label: "Weekly Challenges", component: WeeklyChallenges },
	{ value: 'DarkSectors', label: "Dark Sectors", component: DarkSectors },
	{ value: 'Kinepage', label: "Kinepage", component: Kinepage },
	{ value: 'Faceoff', label: "Faceoff Bonus", component: Faceoff },
	{ value: 'SteelPath', label: "Steel Path", component: SteelPath },
	{ value: 'PrimeResurgence', label: "Prime Resurgence", component: PrimeResurgence },
	{ value: 'ClanInitiative', label: "Clan Initiative", component: ClanInitiative },
	{ value: 'CommunityCampaign', label: "Community Campaign", component: CommunityCampaign },
] as const satisfies readonly {
	value: string;
	label: string;
	component: Component<DashboardViewProps>;
	itemCount?: (world: WorldState, now: number) => number;
	statusLabel?: (world: WorldState, now: number) => string | undefined;
}[];

export type DashboardView = (typeof dashboardViews)[number]['value'];

const viewGroups = [
	{
		label: 'News & Events',
		views: ['news', 'Events', 'Kinepage', 'ClanInitiative', 'CommunityCampaign'],
	},
	{
		label: 'Missions',
		views: ['fissures', 'Alerts', 'Invasions', 'SyndicateMissions', 'KuvaMissions', 'DarkSectors'],
	},
	{
		label: 'Challenges',
		views: ['Sortie', 'ArchonHunt', 'Arbitration', 'Circuit', 'Archimedea', 'Descendia', 'Nightwave', 'WeeklyChallenges', 'Conclave', 'Faceoff'],
	},
	{
		label: 'Vendors & Market',
		views: ['BaroInventory', 'TenetWeapons', 'CodaWeapons', 'Acrithis', 'SteelPath', 'PrimeResurgence', 'DailyDeals', 'FlashSales'],
	},
	{
		label: 'World Activity',
		views: ['WorldCycles', 'Calendar', 'SentientOutposts', 'Simaris', 'PersistentEnemies', 'Construction', 'GlobalUpgrades'],
	},

] satisfies { label: string; views: DashboardView[] }[];

export const dashboardViewGroups = viewGroups.map((group) => ({
	label: group.label,
	views: group.views.map((value) => dashboardViews.find((view) => view.value === value)!),
}));
