<script lang="ts">
	import { names } from '$lib/stores/NameList';
	import { pairs, addPair, removePair } from '$lib/stores/ForbiddenPairs';

	let selectedIndex1: number | null = null;
	let selectedIndex2: number | null = null;
	let disableAddButton: boolean = true;

	$: disableAddButton =
		selectedIndex1 === null || selectedIndex2 === null || selectedIndex1 === selectedIndex2;
</script>

<div class="forbidden-pairs-manager">
	<div class="pair-selector">
		<h3 class="section-title">Add Forbidden Pairings</h3>
		<p class="section-description">
			Prevent specific participants from being assigned to each other
		</p>

		<div class="selector-grid">
			<select
				class="form-select"
				bind:value={selectedIndex1}
				aria-label="Select first participant for forbidden pairing"
			>
				<option value={null} disabled selected>First participant</option>
				{#each $names as name, index}
					<option value={index}>{name}</option>
				{/each}
			</select>

			<div class="separator-icon" aria-hidden="true">↔</div>

			<select
				class="form-select"
				bind:value={selectedIndex2}
				aria-label="Select second participant for forbidden pairing"
			>
				<option value={null} disabled selected>Second participant</option>
				{#each $names as name, index}
					<option value={index}>{name}</option>
				{/each}
			</select>
		</div>

		<button
			class="btn btn-primary"
			on:click={() =>
				selectedIndex1 !== null &&
				selectedIndex2 !== null &&
				addPair(selectedIndex1, selectedIndex2)}
			disabled={disableAddButton}
			aria-label="Add forbidden pairing"
		>
			Add Forbidden Pairing
		</button>
	</div>

	<div class="forbidden-list" role="list" aria-label="Forbidden pairings list">
		{#if $pairs.length === 0}
			<div class="empty-state">
				<p class="empty-message">No forbidden pairings</p>
				<p class="empty-hint">All participants can be assigned to each other</p>
			</div>
		{:else}
			<div class="list-header">
				<h3 class="list-title">Forbidden Pairings ({$pairs.length})</h3>
			</div>
			<ul class="pair-items">
				{#each $pairs as [index1, index2], idx}
					<li class="pair-item" role="listitem">
						<div class="pair-content">
							<span class="pair-name">{$names[index1]}</span>
							<span class="pair-separator">↔</span>
							<span class="pair-name">{$names[index2]}</span>
						</div>
						<button
							class="btn btn-icon btn-danger"
							on:click={() => removePair(idx)}
							aria-label="Remove forbidden pairing between {$names[index1]} and {$names[index2]}"
						>
							×
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
</div>

<style>
	.forbidden-pairs-manager {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		width: 100%;
	}

	.pair-selector {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.section-title {
		margin: 0;
		font-size: var(--font-size-lg);
		font-weight: 600;
		color: var(--color-text);
	}

	.section-description {
		margin: 0;
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.selector-grid {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		gap: var(--space-3);
		align-items: center;
		margin-bottom: var(--space-3);
	}

	@media (max-width: 640px) {
		.selector-grid {
			grid-template-columns: 1fr;
		}

		.separator-icon {
			display: none;
		}
	}

	.separator-icon {
		font-size: var(--font-size-lg);
		color: var(--color-text-muted);
		text-align: center;
	}

	.forbidden-list {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-lg);
		overflow: hidden;
		min-height: 150px;
		max-height: 300px;
		display: flex;
		flex-direction: column;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-6);
		text-align: center;
		flex: 1;
	}

	.empty-message {
		color: var(--color-text);
		font-size: var(--font-size-base);
		margin: 0 0 var(--space-2) 0;
	}

	.empty-hint {
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
		margin: 0;
	}

	.list-header {
		padding: var(--space-3) var(--space-4);
		border-bottom: 1px solid var(--color-border);
		background: var(--color-background);
	}

	.list-title {
		margin: 0;
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-muted);
	}

	.pair-items {
		list-style: none;
		margin: 0;
		padding: 0;
		overflow-y: auto;
		flex: 1;
	}

	.pair-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-4);
		border-bottom: 1px solid var(--color-border);
		transition: background-color 0.15s ease;
	}

	.pair-item:hover {
		background-color: var(--color-hover);
	}

	.pair-item:last-child {
		border-bottom: none;
	}

	.pair-content {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex: 1;
	}

	.pair-name {
		font-size: var(--font-size-base);
		color: var(--color-text);
	}

	.pair-separator {
		color: var(--color-text-muted);
		font-weight: 500;
	}

	.btn-icon {
		width: 32px;
		height: 32px;
		padding: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 1.5rem;
		line-height: 1;
		border-radius: var(--radius-md);
	}

	@media (max-width: 640px) {
		.forbidden-list {
			max-height: 250px;
		}
	}
</style>
