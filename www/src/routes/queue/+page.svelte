<script lang="ts">
	import { queue, entityTitle, listType, currentTrack } from '$lib/store';
	import Info from '$lib/components/Info.svelte';
	import List from '$lib/components/List.svelte';
	import ListItem from '$lib/components/ListItem.svelte';

	import { controls } from '$lib/store';
</script>

<div class="flex max-h-full flex-grow flex-col gap-4">
	<div class="p-4 text-center">
		<p class="text-lg">{$entityTitle}</p>
		{#if $listType === 'Album' && $currentTrack?.artist?.name}
			<p class="text-lg">by {$currentTrack.artist.name}</p>
		{/if}
	</div>

	<List>
		{#each $queue as track}
			<ListItem>
				<button
					class="flex w-full flex-row gap-x-4 p-4 text-left text-base"
					class:bg-blue-800={track.status === 'Playing'}
					class:text-gray-500={track.status === 'Played'}
					on:click|stopPropagation={() => $controls.skipTo(track.position)}
				>
					{#if $listType === 'Album' || $listType === 'Track'}
						<span class="self-start">{track.number.toString().padStart(2, '0')}</span>
					{:else if $listType === 'Playlist'}
						<span>{track.position.toString().padStart(2, '0')}</span>
					{/if}
					<div class="flex flex-grow items-center justify-between overflow-hidden">
						<span class="truncate">
							{track.title}
						</span>
						<Info explicit={track.explicit} hiresAvailable={track.hiresAvailable} />
					</div>
				</button>
			</ListItem>
		{/each}
	</List>
</div>
