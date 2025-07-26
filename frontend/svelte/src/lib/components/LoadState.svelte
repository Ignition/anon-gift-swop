<script lang="ts">
	import { onMount } from 'svelte';
	import { names } from '$lib/stores/NameList';
	import { selection } from '$lib/stores/Selection';
	import { pairs } from '$lib/stores/ForbiddenPairs';
	import { history } from '$lib/stores/AssignmentHistory';
	import { urlLoadingState } from '$lib/stores/LoadingState';
	import { loadingUtils, type AppState } from '$lib/utils';
	import { compactEncoding } from '$lib/compactEncoding';
	import LoadingSpinner from './LoadingSpinner.svelte';
	import init from '$lib/wasm/rust_wasm.js';
	import { devConsole } from '$lib/devConsole';

	let wasmInitialized = false;

	function updateStoresFromParams(params: Partial<AppState>): void {
		if (params.names) {
			names.update(() => params.names!);
		}
		if (params.selection) {
			selection.update(() => new Set(params.selection));
		}
		if (params.forbiddenPairs) {
			// Convert back to tuple format expected by the store
			const pairTuples = params.forbiddenPairs.map(
				(pair) => [pair.index1, pair.index2] as [number, number]
			);
			pairs.update(() => pairTuples);
		}
		if (params.assignmentHistory) {
			history.update(() => params.assignmentHistory!);
		}
	}

	async function loadCompactState(compactParam: string): Promise<Partial<AppState> | null> {
		try {
			const decoded = compactEncoding.decodeState(compactParam);

			// Convert compact format to AppState format
			const appState: Partial<AppState> = {
				names: decoded.names,
				selection: Array.from({ length: decoded.names.length }, (_, i) => i), // Select all by default
				forbiddenPairs: [],
				assignmentHistory: []
			};

			// Convert flattened forbidden pairs back to tuple format
			if (decoded.forbidden_pairs.length > 0) {
				const pairTuples = [];
				for (let i = 0; i < decoded.forbidden_pairs.length; i += 2) {
					pairTuples.push({
						index1: decoded.forbidden_pairs[i],
						index2: decoded.forbidden_pairs[i + 1]
					});
				}
				appState.forbiddenPairs = pairTuples;
			}

			// Convert flattened history back to nested format
			if (decoded.history.length > 0 && decoded.names.length > 0) {
				const historyRounds = [];
				for (let i = 0; i < decoded.history.length; i += decoded.names.length) {
					historyRounds.push(decoded.history.slice(i, i + decoded.names.length));
				}
				appState.assignmentHistory = historyRounds;
			}

			return appState;
		} catch (e) {
			devConsole.log('Compact state decoding failed:', e);
			return null;
		}
	}

	onMount(async (): Promise<void> => {
		urlLoadingState.set(loadingUtils.createLoading('Loading saved state...'));

		// Initialize WASM first
		try {
			await init({});
			wasmInitialized = true;
		} catch (error) {
			devConsole.error('Failed to initialize WASM in LoadState:', error);
		}

		try {
			const urlParams = new URLSearchParams(window.location.search);

			// Try compact state parameter first ('s=')
			const compactParam = urlParams.get('s');
			if (compactParam && wasmInitialized) {
				const decodedData = await loadCompactState(compactParam);
				if (decodedData) {
					updateStoresFromParams(decodedData);
					urlLoadingState.set(loadingUtils.createSuccess());
					return;
				}
			}

			// No state parameters found - this is normal
			urlLoadingState.set(loadingUtils.createSuccess());
		} catch (error) {
			devConsole.error('Error loading state from URL:', error);
			urlLoadingState.set(loadingUtils.createError('Error loading saved state'));
		}
	});
</script>

<LoadingSpinner loadingState={$urlLoadingState} size="small" />

<!-- This component handles URL parameter loading and may show loading states -->
<div aria-hidden="true" class="sr-only">Loading state from URL parameters</div>
