import { writable } from 'svelte/store';

/** @type {import('svelte/store').Writable<[number, number][]>} */
export const pairs = writable([]);

/**
 * @param {number} a
 * @param {number} b
 */
export function addPair(a, b) {
	const sorted = [a, b].sort((x, y) => x - y);
	/** @type {[number, number]} */
	const newPair = [sorted[0], sorted[1]];

	pairs.update((currentArr) => {
		if (currentArr.some((pair) => pair.every((val, idx) => val === newPair[idx]))) {
			return currentArr;
		}
		return [...currentArr, newPair];
	});
}

/**
 * @param {number} index
 */
export function removePair(index) {
	pairs.update((currentArr) => {
		if (index >= 0 && index < currentArr.length) {
			return currentArr.filter((_, i) => i !== index);
		}
		return currentArr;
	});
}
