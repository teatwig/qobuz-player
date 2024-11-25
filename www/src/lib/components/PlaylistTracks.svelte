<script>
	import { playlistTracks, playlistTitle } from '$lib/websocket';
	import List from './List.svelte';
	import ListItem from './ListItem.svelte';
	import PlaylistTrack from './PlaylistTrack.svelte';
	import { writable } from 'svelte/store';
	import { XMark, Icon, Play } from 'svelte-hero-icons';

	export let controls, showPlaylistTracks;

	const show = writable(null);

	const toggle = (id) => {
		if ($show === id) {
			show.set(null);
		} else {
			show.set(id);
		}
	};
</script>

<div class="flex absolute top-0 left-0 flex-col w-full h-full bg-black">
	<div class="flex flex-row justify-between py-4 px-4 bg-black">
		<h2>
			Tracks in <span class="font-bold">{$playlistTitle}</span>
		</h2>
		<div class="flex flex-row flex-nowrap gap-x-2">
			<button
				on:click={() => {
					show.set(null);
					controls.playPlaylist($playlistTracks.id);
				}}
			>
				<Icon src={Play} class="size-6" />
			</button>
			<button
				on:click={() => {
					showPlaylistTracks.set(false);
					show.set(null);
				}}
			>
				<Icon src={XMark} class="size-6" />
			</button>
		</div>
	</div>
	<div class="overflow-y-scroll">
		<List>
			{#each $playlistTracks.tracks as track}
				<ListItem>
					<button class="w-full" on:click={() => toggle(track.id)}>
						<PlaylistTrack {controls} {track} show={$show === track.id} />
					</button>
				</ListItem>
			{/each}
		</List>
	</div>
</div>
