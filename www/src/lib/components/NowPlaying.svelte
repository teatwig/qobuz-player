<script>
	import { queue, entityTitle, listType, currentTrack } from '$lib/websocket';
	import List from './List.svelte';
	import ListItem from './ListItem.svelte';

	export let controls;
</script>

<div class="flex flex-col gap-4 max-h-full">
	<div class="text-center flex-grow-0 p-4">
		<p class="text-xl xl:text-4xl">{$entityTitle}</p>
		{#if $listType === 'Album'}
			<p class="text-xl xl:text-3xl">by {$currentTrack.artist.name}</p>
		{/if}
	</div>

	<List>
		{#each $queue as track}
			<ListItem>
				<button
					class:opacity-60={track.status === 'Played'}
					class:bg-blue-800={track.status === 'Playing'}
					on:click|stopPropagation={() => controls.skipTo(track.position)}
					class="text-base flex flex-row text-left gap-x-4 p-4 w-full"
				>
					{#if $listType === 'Album' || $listType === 'Track'}
						<span class="self-start">{track.number.toString().padStart(2, '0')}</span>
					{:else if $listType === 'Playlist'}
						<span>{track.position.toString().padStart(2, '0')}</span>
					{/if}
					<span>
						{track.title}
					</span>
				</button>
			</ListItem>
		{/each}
	</List>
</div>
