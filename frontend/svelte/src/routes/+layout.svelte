<script lang="ts">
	import '../app.css';
</script>

<div class="app-wrapper">
	<header class="app-header">
		<div class="main-container">
			<h1 class="app-title">🎁 Anon Gift Swop</h1>
			<p class="app-subtitle">Anonymous gift assignment made simple</p>
		</div>
	</header>

	<main class="main-container">
		<slot />
	</main>

	<footer class="app-footer">
		<div class="footer-container">
			<div class="footer-content">
				<p class="footer-text">Created by <strong>Gareth Lloyd</strong></p>
				<div class="social-links">
					<a
						href="https://fosstodon.org/@glloyd"
						target="_blank"
						rel="noopener noreferrer"
						class="social-link mastodon"
						aria-label="Follow Gareth on Mastodon"
					>
						<span class="social-icon">🐘</span>
						<span class="social-text">@glloyd@fosstodon.org</span>
					</a>
					<a
						href="https://www.linkedin.com/in/gareth-lloyd/"
						target="_blank"
						rel="noopener noreferrer"
						class="social-link linkedin"
						aria-label="Connect with Gareth on LinkedIn"
					>
						<span class="social-icon">💼</span>
						<span class="social-text">LinkedIn</span>
					</a>
					<a
						href="https://github.com/Ignition"
						target="_blank"
						rel="noopener noreferrer"
						class="social-link github"
						aria-label="View Gareth's GitHub profile"
					>
						<span class="social-icon">💻</span>
						<span class="social-text">GitHub</span>
					</a>
				</div>
			</div>
		</div>
	</footer>
</div>

<style>
	:global(html, body) {
		margin: 0;
		padding: 0;
		min-height: 100vh;
	}

	.app-wrapper {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}

	.main-container {
		flex: 1;
		width: 100%;
	}
</style>
