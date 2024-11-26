<script>
	import Info from './Info.svelte';

	export let album;
</script>

<div class="flex gap-4 items-center w-full">
	<img loading="lazy" class="size-12 rounded-md" src={album.coverArt} alt={album.title} />

	<div class="overflow-hidden w-full">
		<div class="flex justify-between">
			<h3 class="text-xl truncate">
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
