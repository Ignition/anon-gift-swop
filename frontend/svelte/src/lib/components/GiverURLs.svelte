<script lang="ts">
	import { derived, writable } from 'svelte/store';
	import { onMount } from 'svelte';
	import { assignment } from '$lib/stores/CurrentAssignment';
	import { names } from '$lib/stores/NameList';
	import { selection } from '$lib/stores/Selection';
	import { pairs } from '$lib/stores/ForbiddenPairs';
	import { history } from '$lib/stores/AssignmentHistory';
	import { wasmLoadingState, assignmentLoadingState } from '$lib/stores/LoadingState';
	import { assignmentUtils, urlUtils, loadingUtils } from '$lib/utils';
	import init, { process_assignment } from '$lib/wasm/rust_wasm.js';
	import LoadingSpinner from './LoadingSpinner.svelte';

	const currentUrl = writable<string>('');

	onMount(async () => {
		try {
			wasmLoadingState.set(loadingUtils.createLoading('Loading WebAssembly module...'));
			await init();
			currentUrl.update(() => urlUtils.getBaseUrl());
			wasmLoadingState.set(loadingUtils.createSuccess());
		} catch (error) {
			console.error('Failed to initialize WASM:', error);
			wasmLoadingState.set(loadingUtils.createError('Failed to load WebAssembly module'));
		}
	});

	interface AssignmentName {
		giver: string;
		reciever: string;
		url: string;
	}

	const assignment_names = derived(assignment, ($assignment): AssignmentName[] => {
		const result: AssignmentName[] = [];

		if (Array.isArray($assignment)) {
			const length = $assignment.length;

			// To not expose the assignment in the ordering we need to randomise
			// the display order
			const shuffledIndices = Array.from({ length }, (_, i) => i)
				.map((value) => ({ value, sort: Math.random() }))
				.sort((a, b) => a.sort - b.sort)
				.map(({ value }) => value);

			for (let i = 0; i < length; i++) {
				const currentIndex = shuffledIndices[i];
				const nextIndex = (currentIndex + 1) % length;
				const giver = $names[$assignment[currentIndex]];
				const reciever = $names[$assignment[nextIndex]];
				result.push({
					giver,
					reciever,
					url: createMessageUrl($currentUrl, giver, reciever)
				});
			}
		}
		return result;
	});

	function createMessageUrl(url: string, nameX: string, nameY: string): string {
		return assignmentUtils.createAssignmentUrl(url, nameX, nameY);
	}

	function createAssignment(): void {
		assignmentLoadingState.set(loadingUtils.createLoading('Generating assignment...'));

		try {
			assignment.update(() => {
				if ($selection && $selection.size > 0) {
					const res = process_assignment(Array.from($selection), $pairs, $history);
					if (res) {
						assignmentLoadingState.set(loadingUtils.createSuccess());
						return Array.from(res);
					} else {
						// Calculate the number of constraints
						const participantCount = $selection.size;
						const forbiddenCount = $pairs.filter(
							([a, b]) => $selection.has(a) && $selection.has(b)
						).length;

						let errorMessage = 'Cannot create a valid assignment with the current constraints.\n\n';

						if (forbiddenCount > participantCount / 2) {
							errorMessage += `You have ${forbiddenCount} forbidden pairings for ${participantCount} participants. Either remove some forbidden pairings or add more participants.`;
						} else if ($history.length > 0) {
							errorMessage += `The assignment history (${$history.length} previous rounds) combined with forbidden pairings makes it impossible to create a new unique assignment. Either clear the history, remove some forbidden pairings, or add more participants.`;
						} else {
							errorMessage +=
								'The combination of participants and forbidden pairings makes a valid assignment impossible. Either remove some forbidden pairings or add more participants.';
						}

						assignmentLoadingState.set(loadingUtils.createError(errorMessage));
						return null;
					}
				}
				assignmentLoadingState.set(
					loadingUtils.createError(
						'Please select at least 2 participants to generate an assignment.'
					)
				);
				return null;
			});
		} catch (error) {
			console.error('Error creating assignment:', error);
			assignmentLoadingState.set(
				loadingUtils.createError(
					'A technical error occurred while generating the assignment. Please try again, or refresh the page if the problem persists.'
				)
			);
		}
	}
</script>

<div class="assignment-generator">
	<LoadingSpinner loadingState={$wasmLoadingState} size="small" />

	{#if !$wasmLoadingState.isLoading && !$wasmLoadingState.error}
		<div class="generator-section">
			<h3 class="section-title">Generate Assignment</h3>
			<p class="section-description">
				Create unique links for each participant to see their gift recipient
			</p>

			<button
				class="btn btn-primary btn-large"
				on:click={createAssignment}
				disabled={$assignmentLoadingState.isLoading || $selection.size < 2}
				aria-label="Generate new gift assignment"
			>
				{#if $assignmentLoadingState.isLoading}
					<span class="loading-spinner" aria-hidden="true"></span>
					Generating...
				{:else}
					🎁 Generate Assignment
				{/if}
			</button>

			{#if $selection.size < 2}
				<p class="helper-text">Select at least 2 participants to generate an assignment</p>
			{/if}
		</div>

		<LoadingSpinner loadingState={$assignmentLoadingState} size="medium" />

		{#if $assignment_names.length > 0 && !$assignmentLoadingState.isLoading}
			<div class="assignment-results">
				<div class="results-header">
					<h3 class="section-title">Assignment Links</h3>
					<p class="section-description">Share these unique links with each participant</p>
				</div>

				<div class="link-grid" role="list" aria-label="Gift assignment links">
					{#each $assignment_names as { giver, url }}
						<div class="link-card" role="listitem">
							<div class="link-header">
								<span class="participant-label">For {giver}</span>
							</div>
							<div class="link-content">
								<input
									class="link-input"
									type="text"
									readonly
									value={url}
									aria-label="Assignment link for {giver}"
									on:click={(e) => e.currentTarget.select()}
								/>
								<div class="link-actions">
									<button
										class="btn btn-sm btn-secondary"
										on:click={() => navigator.clipboard.writeText(url)}
										aria-label="Copy link for {giver}"
									>
										📋 Copy
									</button>
									<a
										class="btn btn-sm btn-primary"
										target="_blank"
										href={url}
										aria-label="Open assignment link for {giver}"
										rel="noopener noreferrer"
									>
										🔗 Open
									</a>
								</div>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	{/if}
</div>

<style>
	.assignment-generator {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		width: 100%;
	}

	.generator-section {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-3);
		text-align: center;
		padding: var(--space-6) 0;
	}

	.section-title {
		margin: 0;
		font-size: var(--font-size-xl);
		font-weight: 600;
		color: var(--color-text);
	}

	.section-description {
		margin: 0;
		font-size: var(--font-size-base);
		color: var(--color-text-muted);
		max-width: 500px;
	}

	.btn-large {
		padding: var(--space-3) var(--space-6);
		font-size: var(--font-size-lg);
		gap: var(--space-2);
	}

	.loading-spinner {
		display: inline-block;
		width: 16px;
		height: 16px;
		border: 2px solid rgba(255, 255, 255, 0.3);
		border-radius: 50%;
		border-top-color: white;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.helper-text {
		margin: 0;
		font-size: var(--font-size-sm);
		color: var(--color-warning);
	}

	.assignment-results {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.results-header {
		text-align: center;
	}

	.link-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		gap: var(--space-4);
	}

	@media (max-width: 640px) {
		.link-grid {
			grid-template-columns: 1fr;
		}
	}

	.link-card {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-lg);
		padding: var(--space-4);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		transition:
			transform 0.15s ease,
			box-shadow 0.15s ease;
	}

	.link-card:hover {
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
	}

	.link-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.participant-label {
		font-weight: 600;
		color: var(--color-text);
		font-size: var(--font-size-base);
	}

	.link-content {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.link-input {
		width: 100%;
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		background: var(--color-background);
		font-family: monospace;
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		cursor: pointer;
		transition: border-color 0.15s ease;
	}

	.link-input:hover,
	.link-input:focus {
		border-color: var(--color-primary);
		outline: none;
	}

	.link-actions {
		display: flex;
		gap: var(--space-2);
	}

	.btn-sm {
		padding: var(--space-2) var(--space-3);
		font-size: var(--font-size-sm);
	}

	@media (max-width: 480px) {
		.link-actions {
			flex-direction: column;
		}

		.btn-sm {
			width: 100%;
		}
	}
</style>
