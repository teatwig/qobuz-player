<script>
	import HiRes from '../icons/HiRes.svelte';
	import Explicit from '../icons/Explicit.svelte';

	export let album;
</script>

<div>
	<div class="flex justify-between">
		<h3 class="truncate">
			{album.title}
		</h3>
		<div class="text-gray-400 whitespace-nowrap">
			{#if album.explicit}
				<Explicit />
			{/if}

			{#if album.hiresAvailable}
				<HiRes />
			{/if}
		</div>
	</div>

	<h4 class="flex gap-2 text-left text-gray-400">
		<span>{album.artist.name}</span>
		<span>•︎</span>
		<span>{album.releaseYear}</span>
	</h4>
</div>
