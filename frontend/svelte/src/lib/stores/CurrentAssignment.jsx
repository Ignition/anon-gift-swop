import { writable } from 'svelte/store';

/** @type {import('svelte/store').Writable<number[] | null>} */
export const assignment = writable(null);
