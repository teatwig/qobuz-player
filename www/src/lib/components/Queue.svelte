<script>
	import { queue, entityTitle, listType, currentTrack } from '$lib/websocket';
	import List from './List.svelte';
	import ListItem from './ListItem.svelte';

	export let controls;
</script>

<div class="flex flex-col flex-grow gap-4 max-h-full">
	<div class="p-4 text-center">
		<p class="text-xl">{$entityTitle}</p>
		{#if $listType === 'Album'}
			<p class="text-xl">by {$currentTrack.artist.name}</p>
		{/if}
	</div>

	<List>
		{#each $queue as track}
			<ListItem>
				<button
					class:text-gray-500={track.status === 'Played'}
					class:bg-blue-800={track.status === 'Playing'}
					on:click|stopPropagation={() => controls.skipTo(track.position)}
					class="flex flex-row gap-x-4 p-4 w-full text-base text-left"
				>
					{#if $listType === 'Album' || $listType === 'Track'}
						<span class="self-start">{track.number.toString().padStart(2, '0')}</span>
					{:else if $listType === 'Playlist'}
						<span>{track.position.toString().padStart(2, '0')}</span>
					{/if}
					<span class="truncate">
						{track.title}
					</span>
				</button>
			</ListItem>
		{/each}
	</List>
</div>
