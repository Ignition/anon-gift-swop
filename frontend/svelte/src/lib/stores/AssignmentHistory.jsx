import { writable } from 'svelte/store';

/** @type {import('svelte/store').Writable<number[][]>} */
export const history = writable([]);
