import { writable } from 'svelte/store';
import { loadingUtils, type LoadingState } from '$lib/utils';

export const wasmLoadingState = writable<LoadingState>(
	loadingUtils.createLoading('Initializing WASM module...')
);
export const assignmentLoadingState = writable<LoadingState>(loadingUtils.createSuccess());
export const urlLoadingState = writable<LoadingState>(loadingUtils.createSuccess());
