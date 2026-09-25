import { invoke } from '@tauri-apps/api/core';
import { LazyStore } from '@tauri-apps/plugin-store';

export type MarketStatus = 'invisible' | 'online' | 'ingame';
export type MarketSession = { ingameName: string; slug: string; status: MarketStatus; invisibleAt: number | null };

export function marketProfileUrl(session: MarketSession) {
	return `https://warframe.market/profile/${encodeURIComponent(session.slug)}`;
}

const credentials = new LazyStore('market-account.json');
export const marketAccount = $state<{ session: MarketSession | null; ready: boolean; invisibleOnExit: boolean }>({
	session: null,
	ready: false,
	invisibleOnExit: true,
});

export async function loadMarketSession() {
	try {
		const [session, preference] = await Promise.allSettled([
			invoke<MarketSession | null>('market_session'),
			credentials.get<boolean>('invisible_on_exit'),
		]);
		if (preference.status === 'fulfilled') marketAccount.invisibleOnExit = preference.value ?? true;
		else console.error('Could not load market close preference:', preference.reason);
		if (session.status === 'rejected') throw session.reason;
		marketAccount.session = session.value;
	} finally {
		marketAccount.ready = true;
	}
}

export async function rememberedMarketLogin() {
	return (await credentials.get<{ email: string; password: string }>('login')) ?? null;
}

export async function loginMarket(email: string, password: string, remember: boolean) {
	const session = await invoke<MarketSession>('market_login', { email, password });
	marketAccount.session = session;
	if (remember) await credentials.set('login', { email, password });
	else await credentials.delete('login');
	await credentials.save();
}

export async function logoutMarket() {
	await invoke('market_logout');
	marketAccount.session = null;
}

export async function setMarketStatus(status: MarketStatus) {
	marketAccount.session = await invoke<MarketSession>('market_set_status', { status });
}

export async function scheduleMarketInvisible(minutes: 0 | 30 | 60 | 120 | 240) {
	marketAccount.session = await invoke<MarketSession>('market_schedule_invisible', { minutes });
}

export async function setMarketInvisibleOnExit(enabled: boolean) {
	await credentials.set('invisible_on_exit', enabled);
	await credentials.save();
	marketAccount.invisibleOnExit = enabled;
}
