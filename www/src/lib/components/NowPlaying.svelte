<script>
	import {
		currentTrack,
		numOfTracks,
		entityTitle,
		positionString,
		durationString,
		position,
		coverImage
	} from '$lib/websocket';
	import Info from './Info.svelte';
	import { Backward, Forward, Icon, Pause, Play } from 'svelte-hero-icons';
	import { currentStatus } from '$lib/websocket';
	import Marquee from './Marquee.svelte';

	$: progress = ($position / $currentTrack.durationSeconds) * 100;

	export let controls;
</script>

<div class="flex flex-col gap-8 justify-between items-center p-8 h-full landscape:flex-row">
	<div class="max-h-full rounded-lg shadow-lg overflow-clip aspect-square">
		<img src={$coverImage} alt={$entityTitle} class="object-contain" />
	</div>

	<div class="flex flex-col flex-grow justify-center w-full">
		<div class="flex justify-between items-center">
			<span class="text-xl truncate">
				<Marquee input={$entityTitle} />
			</span>
			<div class="text-gray-500 whitespace-nowrap">
				{$currentTrack.number} of {$numOfTracks}
			</div>
		</div>
		<div class="text-gray-400">
			<Marquee input={$currentTrack?.artist.name} />
		</div>

		<div class="flex flex-col gap-y-4 mx-auto w-full">
			<div class="flex justify-between items-center">
				<span class="text-2xl truncate">
					<Marquee input={$currentTrack?.title} />
				</span>
				<Info explicit={$currentTrack.explicit} hiresAvailable={$currentTrack.hiresAvailable} />
			</div>

			<div>
				<div class="grid h-2 rounded-full overflow-clip">
					<div style="grid-column: 1; grid-row: 1;" class="w-full bg-gray-800"></div>
					<div
						style="grid-column: 1; grid-row: 1;"
						style:width="{progress}%"
						class="bg-gray-500 transition"
					></div>
				</div>
				<div class="flex justify-between text-sm text-gray-500">
					<span>{$positionString}</span>
					<span>{$durationString}</span>
				</div>
			</div>
		</div>

		<div class="flex flex-row gap-2 justify-center h-10">
			<button on:click={() => controls?.previous()}><Icon src={Backward} solid /></button>
			<button on:click={() => controls?.playPause()}>
				{#if $currentStatus === 'Playing'}
					<Icon src={Pause} solid />
				{:else}
					<Icon src={Play} solid />
				{/if}
			</button>
			<button on:click={() => controls?.next()}><Icon src={Forward} solid /></button>
		</div>
	</div>
</div>
