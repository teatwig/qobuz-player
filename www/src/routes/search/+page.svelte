<script lang="ts">
	import { writable } from 'svelte/store';
	import ListAlbums from '$lib/components/ListAlbums.svelte';
	import { Icon, MagnifyingGlass } from 'svelte-hero-icons';
	import ListTracks from '$lib/components/ListTracks.svelte';
	import { onMount } from 'svelte';
	import Spinner from '$lib/components/Spinner.svelte';
	import { controls } from '$lib/store';
	import ListArtists from '$lib/components/ListArtists.svelte';
	import ListPlaylists from '$lib/components/ListPlaylists.svelte';

	const searchTab = writable('albums');

	let query = $state('');

	let abortController: AbortController | undefined;

	let searchResults = $derived.by(() => {
		if (!query.trim()) {
			return {
				query: '',
				albums: [],
				tracks: [],
				artists: [],
				playlists: []
			};
		}

		abortController?.abort();
		abortController = new AbortController();
		return $controls.search(query, abortController);
	});

	let searchInput: HTMLInputElement;
	onMount(() => searchInput.focus());
</script>

<div class="flex max-h-full flex-grow flex-col">
	<div class="flex flex-col gap-4 p-4">
		<form class="flex flex-row items-center gap-4">
			<input
				bind:value={query}
				bind:this={searchInput}
				name="query"
				class="w-full rounded p-2 text-black"
				autocapitalize="off"
				autocomplete="off"
				autocorrect="off"
				placeholder="Search"
				spellcheck="false"
				type="text"
			/>
			<Icon class="size-8" solid src={MagnifyingGlass} />
		</form>

		<div class="flex justify-between *:rounded-full *:px-2 *:py-1 *:transition-colors">
			<button class:bg-blue-800={$searchTab === 'albums'} onclick={() => searchTab.set('albums')}>
				Albums
			</button>
			<button class:bg-blue-800={$searchTab === 'artists'} onclick={() => searchTab.set('artists')}>
				Artists
			</button>
			<button class:bg-blue-800={$searchTab === 'tracks'} onclick={() => searchTab.set('tracks')}>
				Tracks
			</button>
			<button
				class:bg-blue-800={$searchTab === 'playlists'}
				onclick={() => searchTab.set('playlists')}
			>
				Playlists
			</button>
		</div>
	</div>
	{#await searchResults}
		<div class="flex w-full justify-center p-4">
			<Spinner />
		</div>
	{:then data}
		{#if $searchTab === 'albums'}
			<ListAlbums sortBy="default" albums={data.albums} />
		{:else if $searchTab === 'artists'}
			<ListArtists sortBy="default" artists={data.artists} />
		{:else if $searchTab === 'tracks'}
			<ListTracks showTrackNumber={false} tracks={data.tracks} />
		{:else if $searchTab === 'playlists'}
			<ListPlaylists sortBy="default" playlists={data.playlists} />
		{/if}
	{/await}
</div>
