<script lang="ts">
	import Info from './Info.svelte';

	export let album;
</script>

<div class="flex w-full items-center gap-4">
	<img
		class="aspect-square size-12 rounded-md bg-gray-800 text-sm text-gray-500"
		alt={album.title}
		loading="lazy"
		src={album.coverArt}
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
</div>
