<script lang="ts">
	import type { Playlist } from '$lib/bindings/Playlist';
	import List from './List.svelte';
	import ListItem from './ListItem.svelte';

	let { playlists, sortBy }: { playlists: Playlist[]; sortBy: 'default' | 'title' } = $props();

	let sorted = (function () {
		switch (sortBy) {
			case 'default':
				return playlists;
			case 'title':
				return playlists.sort((a, b) => (a.title < b.title ? -1 : 1));
		}
	})();
</script>

<List>
	{#each sorted as playlist}
		<ListItem>
			<a class="flex w-full items-center gap-4 text-left text-lg" href="/playlist?id={playlist.id}">
				<img
					class="aspect-square size-12 rounded-md bg-gray-800 text-sm text-gray-500"
					alt={playlist.title}
					loading="lazy"
					src={playlist.coverArt}
				/>
				<span class="w-full overflow-hidden truncate">
					{playlist.title}
				</span>
			</a>
		</ListItem>
	{/each}
</List>
