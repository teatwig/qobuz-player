<script lang="ts">
	import { userPlaylists, playlistTracks, playlistTitle } from '$lib/websocket';
	import { writable } from 'svelte/store';
	import List from './List.svelte';
	import ListItem from './ListItem.svelte';
	import PlaylistTracks from './PlaylistTracks.svelte';

	export let controls;

	const showPlaylistTracks = writable(false);
</script>

<div class="flex max-h-full flex-grow flex-col gap-4">
	<p class="p-4 text-center text-lg">Playlists</p>
	<List>
		{#each $userPlaylists as playlist}
			<ListItem>
				<button
					class="w-full truncate p-4 text-center"
					on:click|stopPropagation={() => {
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
	</List>

	{#if $showPlaylistTracks}
		<PlaylistTracks {controls} {showPlaylistTracks} />
	{/if}
</div>
