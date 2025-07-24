<script lang="ts">
	import { names } from '$lib/stores/NameList';
	import { selection } from '$lib/stores/Selection';

	let name = '';

	function addName() {
		if (name.trim()) {
			let n = $names.length;
			names.update((arr: string[]) => [...arr, name]);
			selection.update((set: Set<number>) => new Set([...set, n]));
			name = ''; // clear input field
		}
	}

	function toggleSelection(index: number, checked: boolean) {
		selection.update((set: Set<number>) => {
			const newSet = new Set(set);
			if (checked) {
				newSet.add(index);
			} else {
				newSet.delete(index);
			}
			return newSet;
		});
	}
</script>

<div class="participant-manager">
	<div class="input-group">
		<input
			class="form-input"
			type="text"
			bind:value={name}
			placeholder="Enter participant name"
			aria-label="Participant name"
			on:keypress={(e) => e.key === 'Enter' && addName()}
		/>
		<button class="btn btn-primary" on:click={addName} aria-label="Add participant to list">
			<span class="btn-icon">+</span>
			<span class="btn-text">Add</span>
		</button>
	</div>

	<div class="participant-list" role="list" aria-label="Participants list">
		{#if $names.length === 0}
			<div class="empty-state">
				<p class="empty-message">No participants added yet</p>
				<p class="empty-hint">Add names above to get started</p>
			</div>
		{:else}
			<div class="list-header">
				<h3 class="list-title">
					Participants ({$names.filter((_, i) => $selection.has(i)).length} of {$names.length} selected)
				</h3>
			</div>
			<div class="list-items">
				{#each $names as name, index}
					<label class="participant-item" role="listitem">
						<input
							type="checkbox"
							class="checkbox"
							checked={$selection.has(index)}
							aria-label="Include {name} in assignment"
							on:change={(e) => {
								const target = e.currentTarget;
								if (target instanceof HTMLInputElement) {
									toggleSelection(index, target.checked);
								}
							}}
						/>
						<span class="participant-name">{name}</span>
					</label>
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	.participant-manager {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		width: 100%;
	}

	.input-group {
		display: flex;
		gap: var(--space-2);
		width: 100%;
	}

	@media (max-width: 480px) {
		.input-group {
			flex-direction: column;
		}

		.btn-text {
			display: inline;
		}
	}

	.btn-icon {
		font-size: 1.25rem;
		font-weight: bold;
		line-height: 1;
	}

	@media (min-width: 481px) {
		.btn-text {
			display: none;
		}
	}

	.participant-list {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-lg);
		overflow: hidden;
		min-height: 200px;
		max-height: 400px;
		display: flex;
		flex-direction: column;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-8);
		text-align: center;
		flex: 1;
	}

	.empty-message {
		color: var(--color-text);
		font-size: var(--font-size-lg);
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

	.list-items {
		overflow-y: auto;
		flex: 1;
	}

	.participant-item {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		border-bottom: 1px solid var(--color-border);
		cursor: pointer;
		transition: background-color 0.15s ease;
	}

	.participant-item:hover {
		background-color: var(--color-hover);
	}

	.participant-item:last-child {
		border-bottom: none;
	}

	.participant-name {
		flex: 1;
		font-size: var(--font-size-base);
		color: var(--color-text);
	}

	.checkbox {
		width: 20px;
		height: 20px;
		cursor: pointer;
	}

	@media (max-width: 640px) {
		.participant-list {
			max-height: 300px;
		}

		.participant-item {
			padding: var(--space-4);
		}
	}
</style>
