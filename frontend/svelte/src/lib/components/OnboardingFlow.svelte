<script lang="ts">
	import { writable } from 'svelte/store';

	export let showOnboarding = true;

	const currentStep = writable(0);
	const totalSteps = 5;

	const steps = [
		{
			title: '🎁 Welcome to Anonymous Gift Swap!',
			content:
				"This tool helps you organize gift exchanges where participants don't know who's giving to whom until they receive their assignment.",
			action: 'Get Started'
		},
		{
			title: '👥 Step 1: Add Participants',
			content:
				'Add the names of everyone participating in the gift exchange. You can add as many people as you like, but you need at least 3 for a meaningful exchange.',
			action: 'Next: Forbidden Pairings'
		},
		{
			title: '🚫 Step 2: Set Forbidden Pairings (Optional)',
			content:
				"Sometimes you don't want certain people giving to each other (like spouses or family members). Use this section to prevent specific pairings.",
			action: 'Next: Generate Assignment'
		},
		{
			title: '🎯 Step 3: Generate Assignment',
			content:
				'Once you\'re happy with your participants and restrictions, click "Generate Assignment" to create the gift exchange assignments.',
			action: 'Next: Share Links'
		},
		{
			title: '🔗 Step 4: Share Individual Links',
			content:
				"Each participant gets their own private link that tells them who they're giving to. Share these links privately with each person.",
			action: 'Start Using the App'
		}
	];

	function nextStep() {
		if ($currentStep < totalSteps - 1) {
			currentStep.update((n) => n + 1);
		} else {
			showOnboarding = false;
			try {
				localStorage.setItem('gift-swap-onboarding-seen', 'true');
			} catch {
				console.log('Could not save onboarding state to localStorage');
			}
		}
	}

	function skipOnboarding() {
		showOnboarding = false;
		try {
			localStorage.setItem('gift-swap-onboarding-seen', 'true');
		} catch {
			console.log('Could not save onboarding state to localStorage');
		}
	}

	function goToStep(step: number) {
		currentStep.set(step);
	}
</script>

{#if showOnboarding}
	<div
		class="onboarding-overlay"
		role="dialog"
		aria-modal="true"
		aria-labelledby="onboarding-title"
	>
		<div class="onboarding-container">
			<div class="onboarding-header">
				<h2 id="onboarding-title" class="onboarding-title">
					{steps[$currentStep].title}
				</h2>
				<button class="close-button" on:click={skipOnboarding} aria-label="Close onboarding">
					✕
				</button>
			</div>

			<div class="onboarding-content">
				<p class="step-content">{steps[$currentStep].content}</p>

				{#if $currentStep === 0}
					<div class="feature-list">
						<div class="feature-item">
							<span class="feature-icon">🔒</span>
							<span>Completely anonymous assignments</span>
						</div>
						<div class="feature-item">
							<span class="feature-icon">🌐</span>
							<span>Works entirely in your browser - no data stored</span>
						</div>
						<div class="feature-item">
							<span class="feature-icon">📱</span>
							<span>Mobile-friendly and easy to share</span>
						</div>
					</div>
				{/if}
			</div>

			<div class="onboarding-footer">
				<div class="step-indicators">
					<!-- eslint-disable-next-line @typescript-eslint/no-unused-vars -->
					{#each Array(totalSteps) as _, i}
						<button
							class="step-indicator {i === $currentStep ? 'active' : ''}"
							on:click={() => goToStep(i)}
							aria-label="Go to step {i + 1}"
						/>
					{/each}
				</div>

				<div class="onboarding-actions">
					<button class="btn-link skip-button" on:click={skipOnboarding}> Skip Tutorial </button>
					<button class="btn btn-primary next-button" on:click={nextStep}>
						{steps[$currentStep].action}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.onboarding-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: rgba(0, 0, 0, 0.7);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		padding: var(--space-4);
		animation: fadeIn 0.3s ease-out;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	.onboarding-container {
		background: var(--color-surface);
		border-radius: var(--radius-xl);
		max-width: 500px;
		width: 100%;
		max-height: 90vh;
		overflow-y: auto;
		box-shadow: var(--shadow-xl);
		animation: slideUp 0.4s ease-out;
	}

	@keyframes slideUp {
		from {
			opacity: 0;
			transform: translateY(20px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.onboarding-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-6) var(--space-6) var(--space-4);
		border-bottom: 1px solid var(--color-border);
		background: linear-gradient(135deg, var(--color-primary) 0%, var(--color-purple-dark) 100%);
		color: var(--color-white);
	}

	.onboarding-title {
		margin: 0;
		color: var(--color-white);
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
	}

	.close-button {
		background: rgba(255, 255, 255, 0.1);
		border: 1px solid rgba(255, 255, 255, 0.2);
		font-size: 1.5rem;
		cursor: pointer;
		color: var(--color-white);
		padding: var(--space-1);
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-md);
		transition: all 0.2s ease;
	}

	.close-button:hover {
		background-color: rgba(255, 255, 255, 0.2);
	}

	.onboarding-content {
		padding: var(--space-6);
	}

	.step-content {
		font-size: var(--font-size-base);
		line-height: 1.6;
		color: var(--color-text);
		margin: 0 0 var(--space-4) 0;
	}

	.feature-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.feature-item {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3);
		background-color: var(--color-accent-blue);
		background: linear-gradient(135deg, rgba(136, 204, 238, 0.1) 0%, rgba(68, 170, 153, 0.1) 100%);
		border: 1px solid rgba(68, 170, 153, 0.2);
		border-radius: var(--radius-md);
		transition: transform 0.2s ease;
	}

	.feature-item:hover {
		border-color: var(--color-accent-teal);
	}

	.feature-icon {
		font-size: 1.2em;
		filter: saturate(1.5);
	}

	.onboarding-footer {
		padding: var(--space-4) var(--space-6) var(--space-6);
		border-top: 1px solid var(--color-border);
		background-color: var(--color-hover);
	}

	.step-indicators {
		display: flex;
		justify-content: center;
		gap: var(--space-2);
		margin-bottom: var(--space-4);
	}

	.step-indicator {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		border: none;
		background-color: var(--color-gray-300);
		cursor: pointer;
		transition: all 0.3s ease;
	}

	.step-indicator.active {
		background-color: var(--color-primary);
		width: 24px;
		border-radius: 6px;
	}

	.step-indicator:hover:not(.active) {
		background-color: var(--color-accent-teal);
		transform: scale(1.2);
	}

	.onboarding-actions {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-3);
	}

	.skip-button {
		color: var(--color-text-secondary);
		text-decoration: none;
		cursor: pointer;
		border: none;
		background: none;
		font-size: var(--font-size-base);
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-md);
		transition: all 0.2s ease;
	}

	.skip-button:hover {
		background-color: var(--color-hover);
		color: var(--color-text);
	}

	.next-button {
		min-width: 140px;
	}

	/* Mobile responsiveness */
	@media (max-width: 480px) {
		.onboarding-overlay {
			padding: var(--space-3);
		}

		.onboarding-container {
			max-height: 95vh;
		}

		.onboarding-header {
			padding: var(--space-4);
		}

		.onboarding-content,
		.onboarding-footer {
			padding: var(--space-4);
		}

		.onboarding-title {
			font-size: var(--font-size-lg);
		}

		.onboarding-actions {
			flex-direction: column-reverse;
			gap: var(--space-3);
			width: 100%;
		}

		.skip-button,
		.next-button {
			width: 100%;
			justify-content: center;
		}
	}

	/* Accessibility */
	@media (prefers-reduced-motion: reduce) {
		.onboarding-overlay,
		.onboarding-container,
		.step-indicator,
		.close-button,
		.feature-item {
			animation: none;
			transition: none;
		}
	}

	/* Dark mode adjustments */
	@media (prefers-color-scheme: dark) {
		.onboarding-header {
			background: linear-gradient(135deg, var(--color-purple-dark) 0%, var(--color-primary) 100%);
		}

		.feature-item {
			background: linear-gradient(
				135deg,
				rgba(136, 204, 238, 0.05) 0%,
				rgba(68, 170, 153, 0.05) 100%
			);
			border-color: rgba(68, 170, 153, 0.3);
		}
	}
</style>
