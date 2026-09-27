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
