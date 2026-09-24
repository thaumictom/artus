import { invoke } from '@tauri-apps/api/core';
import { LazyStore } from '@tauri-apps/plugin-store';

export type MarketStatus = 'invisible' | 'online' | 'ingame';
export type MarketSession = { ingameName: string; slug: string; status: MarketStatus };

export function marketProfileUrl(session: MarketSession) {
	return `https://warframe.market/profile/${encodeURIComponent(session.slug)}`;
}

const credentials = new LazyStore('market-account.json');
export const marketAccount = $state<{ session: MarketSession | null; ready: boolean }>({
	session: null,
	ready: false,
});

export async function loadMarketSession() {
	try {
		marketAccount.session = await invoke<MarketSession | null>('market_session');
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
