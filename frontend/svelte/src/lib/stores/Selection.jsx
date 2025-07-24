import { writable } from 'svelte/store';

/** @type {import('svelte/store').Writable<Set<number>>} */
export const selection = writable(new Set());
