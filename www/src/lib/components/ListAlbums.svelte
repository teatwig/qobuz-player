<script lang="ts">
	import type { Album } from '$lib/bindings/Album';
	import Info from './Info.svelte';
	import List from './List.svelte';
	import ListItem from './ListItem.svelte';

	let { albums, sortBy }: { albums: Album[]; sortBy: 'default' | 'artist' | 'releaseYear' } =
		$props();

	let sorted = (function () {
		switch (sortBy) {
			case 'default':
				return albums;
			case 'artist':
				return albums.sort((a, b) => (a.artist.name < b.artist.name ? -1 : 1));
			case 'releaseYear':
				return albums.sort((a, b) => (a.releaseYear > b.releaseYear ? -1 : 1));
		}
	})();
</script>

<List>
	{#each sorted as album}
		<ListItem>
			<a class="flex w-full items-center gap-4" href="/album?id={album.id}">
				<img
					class="aspect-square size-12 rounded-md bg-gray-800 text-sm text-gray-500"
					alt={album.title}
					loading="lazy"
					src={album.coverArtSmall}
				/>

				<div class="w-full overflow-hidden">
					<div class="flex justify-between">
						<h3 class="truncate text-lg">
							{album.title}
						</h3>
						<Info explicit={album.explicit} hiresAvailable={album.hiresAvailable} />
					</div>

					<h4 class="flex gap-2 text-left text-gray-400">
						<span class="truncate">{album.artist.name}</span>
						<span>•︎</span>
						<span>{album.releaseYear}</span>
					</h4>
				</div>
			</a>
		</ListItem>
	{/each}
</List>
