<script lang="ts">
	import { playlistTracks, artistAlbums, playlistTitle, Controls } from '$lib/websocket';
	import { writable } from 'svelte/store';
	import ListItem from './ListItem.svelte';
	import ListAlbum from './ListAlbum.svelte';
	import List from './List.svelte';
	import PlaylistTracks from './PlaylistTracks.svelte';
	import { Icon, MagnifyingGlass, XMark } from 'svelte-hero-icons';
	import ListTrack from './ListTrack.svelte';
	import { onMount } from 'svelte';
	import Spinner from './Spinner.svelte';

	let { controls } = $props<{ controls: Controls }>();

	const searchTab = writable('albums');
	const artistName = writable('');
	const showArtistAlbums = writable(false);

	const showPlaylistTracks = writable(false);

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
		return controls.search(query, abortController);
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
				Playlist
			</button>
		</div>
	</div>
	{#await searchResults}
		<div class="flex w-full justify-center p-4">
			<Spinner />
		</div>
	{:then data}
		<List>
			{#if $searchTab === 'albums'}
				{#each data.albums as album}
					<ListItem>
						<button class="w-full p-4 text-left" onclick={() => controls.playAlbum(album.id)}>
							<ListAlbum {album} />
						</button>
					</ListItem>
				{/each}
			{:else if $searchTab === 'artists'}
				{#each data.artists as artist}
					<ListItem>
						<button
							class="w-full truncate p-4 text-left text-lg"
							onclick={() => {
								$artistAlbums.albums = [];
								$artistAlbums.id = null;
								artistName.set(artist.name);
								controls.fetchArtistAlbums(artist.id);
								showArtistAlbums.set(true);
							}}
						>
							{artist.name}
						</button>
					</ListItem>
				{/each}
			{:else if $searchTab === 'tracks'}
				{#each data.tracks as track}
					<ListItem>
						<button class="w-full p-4 text-left" onclick={() => controls.playTrack(track.id)}>
							<ListTrack {track} />
						</button>
					</ListItem>
				{/each}
			{:else if $searchTab === 'playlists'}
				{#each data.playlists as playlist}
					<ListItem>
						<button
							class="w-full truncate p-4 text-left text-lg"
							onclick={() => {
								$playlistTracks.tracks = [];
								$playlistTracks.id = null;
								playlistTitle.set(playlist.title);
								controls.fetchPlaylistTracks(playlist.id);
								showPlaylistTracks.set(true);
							}}
						>
							{playlist.title}
						</button>
					</ListItem>
				{/each}
			{/if}
		</List>
	{/await}

	{#if $showArtistAlbums}
		<div class="absolute left-0 top-0 flex h-full w-full flex-col bg-black">
			<div class="flex flex-row justify-between bg-black px-4 py-4">
				<h2>Albums by <span class="font-bold">{$artistName}</span></h2>
				<button onclick={() => showArtistAlbums.set(false)}
					><Icon class="size-6" src={XMark} /></button
				>
			</div>
			<div class="overflow-y-scroll">
				<List>
					{#each $artistAlbums.albums as album}
						<ListItem>
							<button class="w-full p-4 text-left" onclick={() => controls.playAlbum(album.id)}>
								<ListAlbum {album} />
							</button>
						</ListItem>
					{/each}
				</List>
			</div>
		</div>
	{/if}

	{#if $showPlaylistTracks}
		<PlaylistTracks {controls} {showPlaylistTracks} />
	{/if}
</div>
