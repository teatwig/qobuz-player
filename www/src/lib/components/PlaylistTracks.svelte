<script lang="ts">
	import { playlistTracks, playlistTitle } from '$lib/websocket';
	import List from './List.svelte';
	import ListItem from './ListItem.svelte';
	import ListTrack from './ListTrack.svelte';
	import { XMark, Icon, Play } from 'svelte-hero-icons';

	export let controls, showPlaylistTracks;
</script>

<div class="absolute left-0 top-0 flex h-full w-full flex-col bg-black">
	<div class="flex flex-row justify-between bg-black px-4 py-4">
		<h2>
			Tracks in <span class="font-bold">{$playlistTitle}</span>
		</h2>
		<div class="flex flex-row flex-nowrap gap-x-2">
			<button
				on:click={() => {
					controls.playPlaylist($playlistTracks.id);
				}}
			>
				<Icon class="size-6" src={Play} />
			</button>
			<button
				on:click={() => {
					showPlaylistTracks.set(false);
				}}
			>
				<Icon class="size-6" src={XMark} />
			</button>
		</div>
	</div>
	<div class="overflow-y-scroll">
		<List>
			{#each $playlistTracks.tracks as track}
				<ListItem>
					<button class="w-full p-4 text-left" on:click={() => controls.playTrack(track.id)}>
						<ListTrack {track} />
					</button>
				</ListItem>
			{/each}
		</List>
	</div>
</div>
