<script lang="ts">
	import { onMount } from 'svelte';
	import { derived, writable } from 'svelte/store';
	import { names } from '$lib/stores/NameList';
	import { selection } from '$lib/stores/Selection';
	import { pairs } from '$lib/stores/ForbiddenPairs';
	import { history } from '$lib/stores/AssignmentHistory';
	import { assignment } from '$lib/stores/CurrentAssignment';
	import { versionedUrlUtils, urlUtils, type AppState } from '$lib/utils';

	let href = writable<string>('http://unknown');

	function updateEncodedUrl(assignment?: number[]): string {
		const data: AppState = {
			names: $names || [],
			selection: $selection ? Array.from($selection) : [],
			forbiddenPairs: $pairs
				? $pairs.map(([index1, index2]: [number, number]) => ({ index1, index2 }))
				: [],
			assignmentHistory: assignment ? [assignment, ...$history] : $history || []
		};

		try {
			return versionedUrlUtils.createVersionedStateUrl($href, data);
		} catch (error) {
			console.error('Failed to create versioned state URL:', error);
			return '';
		}
	}

	onMount(() => {
		href.update(() => urlUtils.getCurrentUrl());
	});

	let encodedUrl = derived([href, names, selection, pairs], () => updateEncodedUrl());
	let encodedUrl2 = derived([href, assignment], (): string | null =>
		$assignment ? updateEncodedUrl($assignment) : null
	);
</script>

<div class="save-container">
	<span class="save-label">Save & Share:</span>
	<div class="save-buttons">
		<a
			target="_blank"
			class="btn btn-secondary"
			href={$encodedUrl}
			aria-label="Save current setup configuration"
			rel="noopener noreferrer"
		>
			💾 Setup Only
		</a>
		{#if $assignment}
			<a
				target="_blank"
				class="btn btn-secondary"
				href={$encodedUrl2}
				aria-label="Save current setup with assignment included"
				rel="noopener noreferrer"
			>
				💾 Setup + Assignment
			</a>
		{/if}
	</div>
</div>

<style>
	.save-container {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-wrap: wrap;
	}

	.save-label {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		font-weight: 500;
	}

	.save-buttons {
		display: flex;
		gap: var(--space-2);
		flex-wrap: wrap;
	}

	@media (max-width: 640px) {
		.save-container {
			flex-direction: column;
			align-items: stretch;
			width: 100%;
		}

		.save-buttons {
			flex-direction: column;
			width: 100%;
		}

		.save-buttons .btn {
			width: 100%;
			justify-content: center;
		}
	}
</style>
