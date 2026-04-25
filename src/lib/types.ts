import type { Component } from 'svelte';

export type Sections = Record<string, { label: string; icon: string; component: Component }>;
