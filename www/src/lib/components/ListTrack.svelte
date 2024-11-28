<script lang="ts">
	import Info from './Info.svelte';

	export let track;
</script>

<div>
	<div class="flex items-center justify-between">
		<h3 class="truncate text-lg">
			{track.title}
		</h3>
		<Info explicit={track.explicit} hiresAvailable={track.hiresAvailable} />
	</div>

	<h4 class="truncate text-gray-400">
		{track.title}
	</h4>
</div>
