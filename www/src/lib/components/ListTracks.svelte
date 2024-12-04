<script lang="ts">
	import Info from './Info.svelte';
	import List from './List.svelte';
	import ListItem from './ListItem.svelte';
	import type { Track } from '$lib/bindings/Track';
	import { controls, currentTrack } from '$lib/store';
	import { Icon, Play } from 'svelte-hero-icons';

	let { tracks, showTrackNumber }: { tracks: Track[]; showTrackNumber: boolean } = $props();
</script>

<List>
	{#each tracks as track}
		{@const nowPlaying = $currentTrack?.id === track.id}
		<ListItem>
			<button
				class="flex w-full items-center justify-between text-left"
				onclick={() => $controls.playTrack(track.id)}
			>
				<span class="flex items-center gap-4">
					<span class="w-5 text-center">
						{#if nowPlaying}
							<Icon src={Play} class="size-4 text-blue-500" solid />
						{/if}
						{#if showTrackNumber && !nowPlaying}
							<span class="text-gray-400">{track.position}.</span>
						{/if}
					</span>

					<h2 class="truncate">
						{track.title}
					</h2>
				</span>
				<Info explicit={track.explicit} hiresAvailable={track.hiresAvailable} />
			</button>
		</ListItem>
	{/each}
</List>
