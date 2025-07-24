<script lang="ts">
	import type { LoadingState } from '$lib/utils';

	export let loadingState: LoadingState;
	export let size: 'small' | 'medium' | 'large' = 'medium';
</script>

{#if loadingState.isLoading}
	<div class="loading-container" role="status" aria-live="polite">
		<div class="spinner spinner-{size}" aria-hidden="true"></div>
		{#if loadingState.message}
			<span class="loading-message">{loadingState.message}</span>
		{/if}
		<span class="sr-only">Loading...</span>
	</div>
{:else if loadingState.error}
	<div class="error-container" role="alert" aria-live="assertive">
		<span class="error-icon" aria-hidden="true">⚠️</span>
		<span class="error-message">{loadingState.error}</span>
	</div>
{/if}

<style>
	.loading-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-4);
	}

	.spinner {
		border: 3px solid var(--color-gray-200);
		border-top: 3px solid var(--color-primary);
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	.spinner-small {
		width: 20px;
		height: 20px;
		border-width: 2px;
	}

	.spinner-medium {
		width: 40px;
		height: 40px;
	}

	.spinner-large {
		width: 60px;
		height: 60px;
		border-width: 4px;
	}

	@keyframes spin {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}

	.loading-message {
		color: var(--color-text);
		font-size: var(--font-size-base);
		text-align: center;
	}

	.error-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		gap: var(--space-3);
		padding: var(--space-6);
		background-color: var(--color-danger-light);
		border: 2px solid var(--color-danger);
		border-radius: var(--radius-md);
		color: var(--color-danger);
		font-weight: var(--font-weight-medium);
		width: 100%;
		margin: var(--space-4) 0;
	}

	.error-icon {
		font-size: 2rem;
		line-height: 1;
		margin-bottom: var(--space-2);
	}

	.error-message {
		font-size: var(--font-size-base);
		line-height: 1.6;
		color: var(--color-text);
		white-space: pre-line;
		max-width: 600px;
	}

	/* Reduced motion support */
	@media (prefers-reduced-motion: reduce) {
		.spinner {
			animation: none;
			border: 3px solid var(--color-primary);
		}
	}
</style>
