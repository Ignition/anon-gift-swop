<script lang="ts">
	import LoadState from '$lib/components/LoadState.svelte';
	import SaveState from '$lib/components/SaveState.svelte';
	import ScrollableList from '$lib/components/ScrollableList.svelte';
	import PairSelector from '$lib/components/PairSelector.svelte';
	import GiverURLs from '$lib/components/GiverURLs.svelte';
	import OnboardingFlow from '$lib/components/OnboardingFlow.svelte';

	// Check if user has seen onboarding before (persist in localStorage)
	let showOnboarding = true;

	try {
		const hasSeenOnboarding = localStorage.getItem('gift-swap-onboarding-seen');
		if (hasSeenOnboarding === 'true') {
			showOnboarding = false;
		}
	} catch {
		// If localStorage is not available, show onboarding by default
		console.log('localStorage not available, showing onboarding by default');
	}
</script>

{#if showOnboarding}
	<OnboardingFlow bind:showOnboarding />
{/if}

<div class="app-content">
	<div class="section-grid">
		<section class="card">
			<h2 class="card-title">1. Add Participants</h2>
			<ScrollableList />
		</section>

		<section class="card">
			<h2 class="card-title">2. Set Restrictions</h2>
			<PairSelector />
		</section>

		<section class="card full-width">
			<h2 class="card-title">3. Generate & Share</h2>
			<GiverURLs />
		</section>
	</div>

	<div class="utility-bar">
		<SaveState />
		<LoadState />
	</div>
</div>

<style>
	.app-content {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		padding-bottom: var(--space-8);
	}

	.section-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-6);
		width: 100%;
	}

	@media (max-width: 1024px) {
		.section-grid {
			grid-template-columns: 1fr;
		}
	}

	.card {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-xl);
		padding: var(--space-6);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
		transition: box-shadow 0.2s ease;
	}

	.card-title {
		margin: 0 0 var(--space-4) 0;
		font-size: var(--font-size-xl);
		font-weight: 600;
		color: var(--color-text);
	}

	.full-width {
		grid-column: 1 / -1;
	}

	.utility-bar {
		display: flex;
		justify-content: center;
		gap: var(--space-4);
		padding: var(--space-6);
		border-top: 1px solid var(--color-border);
		background: var(--color-background);
		margin-top: var(--space-8);
	}

	@media (max-width: 640px) {
		.card {
			padding: var(--space-4);
		}

		.utility-bar {
			flex-direction: column;
			padding: var(--space-4);
		}
	}
</style>
