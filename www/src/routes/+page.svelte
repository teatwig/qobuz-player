<script>
	import { onMount } from 'svelte';
	import {
		WS,
		currentTrack,
		isBuffering,
		isLoading,
		currentStatus,
		connected,
		coverImage,
		entityTitle
	} from '$lib/websocket';
	import { writable } from 'svelte/store';
	import { dev } from '$app/environment';
	import Navigation from '../lib/components/Navigation.svelte';
	import TrackMetadata from '../lib/components/TrackMetadata.svelte';
	import { Icon, LinkSlash, ArrowPath } from 'svelte-hero-icons';
	import NowPlaying from '../lib/components/NowPlaying.svelte';
	import Search from '../lib/components/Search.svelte';
	import MyPlaylists from '../lib/components/MyPlaylists.svelte';

	let controls;

	const activePage = writable('playing');
	const setPage = (newPage) => activePage.set(newPage);

	onMount(() => {
		controls = new WS(dev);

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

<div class="flex flex-col justify-between h-full">
	<div
		class="flex flex-col md:h-auto sm:py-4 md:py-0 justify-between md:flex-row overflow-auto h-full"
	>
		{#if $activePage == 'playing'}
			<div class="flex flex-col justify-between p-8 h-full items-center">
				<div class="max-w-sm">
					<img src={$coverImage} alt={$entityTitle} class="object-contain rounded-lg shadow-lg" />
				</div>
				{#if $currentTrack}
					<TrackMetadata {controls} />
				{/if}
			</div>
		{/if}

		{#if $activePage == 'search'}
			<Search {controls} />
		{/if}

		{#if $activePage == 'favorites'}
			<MyPlaylists {controls} />
		{/if}

		{#if $activePage == 'queue'}
			<NowPlaying {controls} />
		{/if}
	</div>
	<Navigation {setPage} {activePage} />
</div>

{#if $isBuffering || !$connected || $isLoading}
	<div class="fixed top-8 right-8 z-10 bg-blue-800 p-2 h-12">
		{#if !$connected}
			<Icon src={LinkSlash} solid />
		{:else if $isLoading || $isBuffering}
			<Icon src={ArrowPath} solid class="animate-spin" />
		{/if}
	</div>
{/if}
