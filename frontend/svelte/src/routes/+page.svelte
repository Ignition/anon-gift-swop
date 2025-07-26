<script lang="ts">
	import { page } from '$app/stores';
	import { browser } from '$app/environment';
	import DefaultPage from './DefaultPage.svelte';
	import MessagePage from './MessagePage.svelte';
	import { devConsole } from '$lib/devConsole';

	// Get message parameter 'm'
	$: msg = browser ? $page.url.searchParams.get('m') : null;

	$: {
		devConsole.log('🏠 +page.svelte - browser:', browser);
		devConsole.log('🏠 +page.svelte - msg param:', msg);
		if (browser && $page.url) {
			devConsole.log('🏠 +page.svelte - full URL:', $page.url.toString());
			devConsole.log(
				'🏠 +page.svelte - search params:',
				Object.fromEntries($page.url.searchParams)
			);
		}
	}
</script>

{#if msg}
	<MessagePage message={msg} />
{:else}
	<DefaultPage />
{/if}
