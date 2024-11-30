<script lang="ts">
	import { onMount } from 'svelte';
	import {
		Controls,
		currentTrack,
		isBuffering,
		isLoading,
		currentStatus,
		connected
	} from '$lib/websocket';
	import { writable } from 'svelte/store';
	import { dev } from '$app/environment';
	import Navigation from '../lib/components/Navigation.svelte';
	import Queue from '../lib/components/Queue.svelte';
	import { Icon, LinkSlash } from 'svelte-hero-icons';
	import NowPlaying from '../lib/components/NowPlaying.svelte';
	import Search from '../lib/components/Search.svelte';
	import Favorites from '../lib/components/Favorites.svelte';
	import Spinner from '../lib/components/Spinner.svelte';

	let controls: Controls;

	const activePage = writable('nowPlaying');
	const setPage = (newPage: string) => activePage.set(newPage);

	onMount(() => {
		controls = new Controls(dev);

		const onFocus = () => {
			if (!$connected) {
				controls.connect();
			}
		};

		window.addEventListener('focus', onFocus);

		return () => {
			controls.close();
			window.removeEventListener('focus', onFocus);
		};
	});
</script>

<svelte:head>
	<title>hifi.rs: {$currentStatus}</title>
</svelte:head>

<div class="flex h-full flex-col justify-between px-safe pt-safe">
	<div class="flex h-full flex-col justify-between overflow-hidden">
		{#if $activePage == 'nowPlaying' && $currentTrack}
			<NowPlaying {controls} />
		{/if}

		{#if $activePage == 'search'}
			<Search {controls} />
		{/if}

		{#if $activePage == 'favorites'}
			<Favorites {controls} />
		{/if}

		{#if $activePage == 'queue'}
			<Queue {controls} />
		{/if}
	</div>
	<Navigation {activePage} {setPage} />
</div>

{#if $isBuffering || !$connected || $isLoading}
	<div class="fixed right-8 top-8 z-10 size-12 rounded bg-black bg-opacity-20 p-2 backdrop-blur">
		{#if !$connected}
			<Icon solid src={LinkSlash} />
		{:else if $isLoading || $isBuffering}
			<Spinner />
		{/if}
	</div>
{/if}
