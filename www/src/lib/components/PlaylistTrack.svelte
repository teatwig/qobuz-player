<script>
	import HiRes from '../icons/HiRes.svelte';
	import Explicit from '../icons/Explicit.svelte';
	export let track;
</script>

<div>
	<div class="flex justify-between">
		<h3 class="truncate">
			{track.title}
		</h3>
		<div class="text-gray-400 whitespace-nowrap">
			{#if track.explicit}
				<Explicit />
			{/if}

			{#if track.hiresAvailable}
				<HiRes />
			{/if}
		</div>
	</div>

	<h4 class="text-gray-400">{track.artist.name}</h4>
</div>
