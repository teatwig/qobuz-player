<script>
	import { afterUpdate } from 'svelte';
	import {
		currentTrack,
		numOfTracks,
		entityTitle,
		positionString,
		durationString,
		position
	} from '$lib/websocket';
	import { writable } from 'svelte/store';
	import { Backward, Forward, Icon, Pause, Play } from 'svelte-hero-icons';
	import { currentStatus } from '$lib/websocket';

	let titleWidth, titleWrapperWidth;

	const enableMarquee = writable(false);

	afterUpdate(() => {
		if (titleWidth > titleWrapperWidth) {
			enableMarquee.set(true);
		} else {
			enableMarquee.set(false);
		}
	});

	$: progress = ($position / $currentTrack.durationSeconds) * 100;

	export let controls;
</script>

<div class="flex flex-col gap-8 text-lg text-center">
	<div class="flex flex-col items-center gap-2">
		<div>{$entityTitle || ''}</div>
		<div class="text-gray-400">
			{$currentTrack?.artist.name || ''}
		</div>
		<div class="text-gray-500 text-base xl:text-4xl">{$currentTrack.number} of {$numOfTracks}</div>
	</div>

	<div class="flex flex-col gap-y-4 mx-auto text-2xl w-full">
		<div
			bind:offsetWidth={titleWrapperWidth}
			class:justify-center={!$enableMarquee}
			class="flex flex-row overflow-hidden"
		>
			<div
				class:marquee={$enableMarquee}
				class:pl-[50%]={$enableMarquee}
				class="md:py-4 flex flex-row leading-[1.15em] xl:py-8 font-semibold py-2 whitespace-nowrap"
			>
				<span bind:offsetWidth={titleWidth}>
					{$currentTrack?.title || ''}
				</span>
			</div>

			{#if $enableMarquee}
				<div
					class:marquee={$enableMarquee}
					class:pl-[50%]={$enableMarquee}
					class="md:py-4 flex flex-row leading-[1.15em] xl:py-8 font-semibold py-2 whitespace-nowrap"
				>
					{$currentTrack?.title || ''}
				</div>
			{/if}
		</div>

		<div>
			<div class="grid h-2 rounded-full overflow-clip">
				<div style="grid-column: 1; grid-row: 1;" class="w-full bg-gray-600"></div>
				<div
					style="grid-column: 1; grid-row: 1;"
					style:width="{progress}%"
					class=" bg-gray-500"
				></div>
			</div>
			<div class="text-gray-500 flex justify-between text-sm">
				<span>{$positionString}</span>
				<span>{$durationString}</span>
			</div>
		</div>
	</div>
	<div class="flex flex-row justify-center gap-2 flex-grow h-10">
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

<style lang="postcss">
	.marquee {
		animation-name: marquee;
		animation-duration: 15s;
		animation-iteration-count: infinite;
		animation-timing-function: linear;
	}

	@keyframes marquee {
		from {
			transform: translateX(0);
		}

		to {
			transform: translateX(-100%);
		}
	}
</style>
