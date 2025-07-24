<script context="module" lang="ts">
	export function load({ params }: { params: Record<string, string> }) {
		return {
			props: {
				message: params.msg || ''
			}
		};
	}
</script>

<script lang="ts">
	import { versionedUrlUtils, type AssignmentPayload } from '$lib/utils';

	export let message: string;

	function decodeMessage(encodedMessage: string): AssignmentPayload | null {
		return versionedUrlUtils.decodeVersionedMessage(encodedMessage);
	}

	const payload = decodeMessage(message);
</script>

{#if payload}
	<div class="message-page" role="main">
		<div class="message-container">
			<div class="message-icon">🎁</div>
			<h1 class="message-title">Your Gift Assignment</h1>
			<div class="main-message">
				<p class="primary-message">
					<strong class="giver-name">{payload.giver}</strong>, you are giving a gift to:
				</p>
				<div class="receiver-highlight">
					<span class="receiver-name">{payload.receiver}</span>
				</div>
				<p class="encouragement">🎁 Time to find the perfect gift!</p>
			</div>
			<div class="message-actions">
				<a href="/" class="btn btn-primary btn-large"> 🚀 Start Your Own Anon Gift Swop </a>
				<p class="cta-subtitle">Organize your own anonymous gift exchange</p>
			</div>
		</div>
	</div>
{:else}
	<div class="message-page" role="main">
		<div class="message-container">
			<div class="message-icon">⚠️</div>
			<h1 class="message-title">Invalid Assignment</h1>
			<div class="message-content">
				<p class="assignment-text">This assignment link appears to be invalid or corrupted.</p>
				<p class="assignment-note">
					Please check the link or contact the person who sent it to you.
				</p>
			</div>
			<div class="message-actions">
				<a href="/" class="btn btn-primary btn-large"> 🚀 Start Your Own Anon Gift Swop </a>
				<p class="cta-subtitle">Organize your own anonymous gift exchange</p>
			</div>
		</div>
	</div>
{/if}

<style>
	.message-page {
		min-height: calc(100vh - 200px);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--space-6) var(--container-padding);
		background: linear-gradient(
			135deg,
			rgba(51, 34, 136, 0.03) 0%,
			rgba(68, 170, 153, 0.03) 50%,
			rgba(136, 204, 238, 0.03) 100%
		);
	}

	.message-container {
		background: var(--color-surface);
		border: 2px solid var(--color-border);
		border-radius: var(--radius-xl);
		padding: var(--space-8) var(--space-6);
		box-shadow: var(--shadow-xl);
		max-width: 800px;
		width: 100%;
		text-align: center;
		position: relative;
		overflow: hidden;
	}

	.message-container::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 4px;
		background: linear-gradient(
			90deg,
			var(--color-primary) 0%,
			var(--color-accent-teal) 50%,
			var(--color-accent-blue) 100%
		);
	}

	.message-icon {
		font-size: 4rem;
		margin-bottom: var(--space-4);
		filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.1));
	}

	.message-title {
		font-size: var(--font-size-3xl);
		font-weight: var(--font-weight-bold);
		color: var(--color-text);
		margin: 0 0 var(--space-6) 0;
		background: linear-gradient(135deg, var(--color-primary), var(--color-accent-teal));
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
	}

	.main-message {
		margin-bottom: var(--space-8);
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		align-items: center;
	}

	.primary-message {
		font-size: var(--font-size-xl);
		color: var(--color-text);
		margin: 0;
		text-align: center;
		line-height: 1.5;
	}

	.giver-name {
		color: var(--color-secondary);
		font-weight: var(--font-weight-bold);
		font-size: var(--font-size-2xl);
	}

	.receiver-highlight {
		background: var(--color-accent-yellow);
		padding: var(--space-6) var(--space-8);
		border-radius: var(--radius-xl);
		box-shadow: 0 4px 16px rgba(221, 204, 119, 0.3);
		border: 3px solid var(--color-warning-hover);
		transform: rotate(-1deg);
		position: relative;
	}

	.receiver-highlight::before {
		content: '🎯';
		position: absolute;
		top: -15px;
		right: -15px;
		font-size: 2rem;
		background: var(--color-white);
		border-radius: 50%;
		width: 50px;
		height: 50px;
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
	}

	.receiver-name {
		font-size: var(--font-size-3xl);
		font-weight: var(--font-weight-bold);
		color: var(--color-text-on-warning);
		text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
		display: block;
		text-align: center;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.encouragement {
		font-size: var(--font-size-lg);
		color: var(--color-text-secondary);
		margin: 0;
		text-align: center;
		font-weight: var(--font-weight-medium);
	}

	.assignment-text {
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-medium);
		color: var(--color-text);
		line-height: 1.5;
		margin: 0 0 var(--space-3) 0;
		text-align: center;
	}

	.message-actions {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-3);
	}

	.btn-large {
		padding: var(--space-4) var(--space-8);
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-semibold);
		gap: var(--space-3);
		box-shadow: var(--shadow-lg);
		transition: all 0.3s ease;
	}

	.btn-large:hover {
		box-shadow: var(--shadow-xl);
		transform: scale(1.02);
	}

	.cta-subtitle {
		margin: 0;
		color: var(--color-text-secondary);
		font-size: var(--font-size-sm);
		font-style: italic;
	}

	/* Mobile Styles */
	@media (max-width: 640px) {
		.message-page {
			min-height: calc(100vh - 160px);
			padding: var(--space-4) var(--space-3);
		}

		.message-container {
			padding: var(--space-8) var(--space-6);
		}

		.message-icon {
			font-size: 3rem;
		}

		.message-title {
			font-size: var(--font-size-2xl);
		}

		.assignment-text {
			font-size: var(--font-size-lg);
		}

		.btn-large {
			width: 100%;
			padding: var(--space-4) var(--space-6);
		}
	}

	/* Accessibility */
	@media (prefers-reduced-motion: reduce) {
		.btn-large:hover {
			transform: none;
		}
	}
</style>
