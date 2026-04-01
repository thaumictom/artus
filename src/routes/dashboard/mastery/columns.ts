import type { ColumnDef } from '@tanstack/table-core';

export type MasterableItem = {
	uniqueName: string;
	name: string;
	category: string;
	masteryReq: number | null;
};

export const columns: ColumnDef<MasterableItem>[] = [
	{
		accessorKey: 'uniqueName',
		header: 'Unique Name',
	},
	{
		accessorKey: 'name',
		header: 'Name',
	},
	{
		accessorKey: 'category',
		header: 'Category',
	},
	{
		accessorKey: 'masteryReq',
		header: 'Mastery Requirement',
	},
];
