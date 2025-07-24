import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [sveltekit()],
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}'],
		environment: 'jsdom',
		// Updated configuration for Vitest v2
		server: {
			deps: {
				inline: [/\.wasm/]
			}
		}
	},
	// Handle WASM files
	assetsInclude: ['**/*.wasm']
});
