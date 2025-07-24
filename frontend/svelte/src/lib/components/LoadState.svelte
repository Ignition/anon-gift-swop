<script lang="ts">
	import { onMount } from 'svelte';
	import { names } from '$lib/stores/NameList';
	import { selection } from '$lib/stores/Selection';
	import { pairs } from '$lib/stores/ForbiddenPairs';
	import { history } from '$lib/stores/AssignmentHistory';
	import { urlLoadingState } from '$lib/stores/LoadingState';
	import { versionedUrlUtils, loadingUtils, type AppState } from '$lib/utils';
	import LoadingSpinner from './LoadingSpinner.svelte';

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

	onMount((): void => {
		urlLoadingState.set(loadingUtils.createLoading('Loading saved state...'));

		try {
			const startWithParam = new URLSearchParams(window.location.search).get('startwith');

			if (startWithParam) {
				const decodedData = versionedUrlUtils.decodeVersionedState(startWithParam);

				if (decodedData) {
					updateStoresFromParams(decodedData);
					urlLoadingState.set(loadingUtils.createSuccess());
				} else {
					urlLoadingState.set(loadingUtils.createError('Failed to load saved state from URL'));
				}
			} else {
				urlLoadingState.set(loadingUtils.createSuccess());
			}
		} catch (error) {
			console.error('Error loading state from URL:', error);
			urlLoadingState.set(loadingUtils.createError('Error loading saved state'));
		}
	});
</script>

<LoadingSpinner loadingState={$urlLoadingState} size="small" />

<!-- This component handles URL parameter loading and may show loading states -->
<div aria-hidden="true" class="sr-only">Loading state from URL parameters</div>
