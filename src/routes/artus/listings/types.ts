export type Listing = {
	id: string;
	itemId: string;
	type: 'buy' | 'sell';
	platinum: number;
	quantity: number;
	visible: boolean;
	perTrade?: number;
	rank?: number;
	subtype?: string;
};

export type ListingItem = { name: string; slug: string };

export type EditableListing = Pick<Listing, 'id' | 'type' | 'platinum' | 'quantity' | 'visible'>;

export type ListingChange =
	| { kind: 'created'; slug: string; platinum: number; visible: boolean; listing: Listing | null }
	| { kind: 'updated'; id: string; slug?: string; platinum: number; quantity: number }
	| { kind: 'visibility'; id: string; slug?: string; visible: boolean }
	| { kind: 'deleted'; id: string; slug?: string };
