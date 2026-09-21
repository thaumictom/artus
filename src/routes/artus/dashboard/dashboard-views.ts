import type { Component } from 'svelte';
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
import SteelPath from './views/SteelPath.svelte';
import PrimeResurgence from './views/PrimeResurgence.svelte';
import ClanInitiative from './views/ClanInitiative.svelte';
import CommunityCampaign from './views/CommunityCampaign.svelte';

export const dashboardViews = [
	{ value: 'fissures', label: 'Fissure Missions', component: FissureMissions },
	{ value: 'news', label: 'News', component: News },
	{ value: 'BaroInventory', label: "Baro Ki'Teer", component: BaroInventory },
	{ value: 'Alerts', label: "Alerts", component: Alerts },
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
] as const satisfies readonly { value: string; label: string; component: Component<DashboardViewProps> }[];

export type DashboardView = (typeof dashboardViews)[number]['value'];
