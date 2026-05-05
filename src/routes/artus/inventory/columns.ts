import type { ColumnDef } from '@tanstack/table-core';

// This type is used to define the shape of our data.
// You can use a Zod schema here if you want.
export type WarframeItem = {
	name: string;
	category?: string;
	quantity: number;
	marketMedian?: number;
	marketMedianUsesOfferFallback?: boolean;
	ducats: number;
};

export const columns: ColumnDef<WarframeItem>[] = [
	{
		accessorKey: 'name',
		header: 'Name',
	},
	{
		accessorKey: 'quantity',
		header: 'Quantity',
	},
	{
		accessorKey: 'marketMedian',
		header: 'Market Median',
	},
	{
		accessorKey: 'ducats',
		header: 'Ducats',
	},
	{
		id: 'totalPrice',
		header: 'Total Price',
		accessorFn: (row) => (row.marketMedian ?? 0) * row.quantity,
	},
	{
		id: 'totalDucats',
		header: 'Total Ducats',
		accessorFn: (row) => row.ducats * row.quantity,
	},
	{
		id: 'actions',
		header: 'Actions',
	},
];
