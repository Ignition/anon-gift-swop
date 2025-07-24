<script lang="ts">
	import { page } from '$app/stores';
	import { browser } from '$app/environment';
	import DefaultPage from './DefaultPage.svelte';
	import MessagePage from './MessagePage.svelte';

	$: msg = browser ? $page.url.searchParams.get('msg') : null;
</script>

{#if msg}
	<MessagePage message={msg} />
{:else}
	<DefaultPage />
{/if}
