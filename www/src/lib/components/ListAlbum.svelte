<script>
	import Info from './Info.svelte';

	export let album;
</script>

<div>
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
