<script>
	import { afterUpdate } from 'svelte';
	import {
		currentTrack,
		numOfTracks,
		entityTitle,
		positionString,
		durationString,
		position,
		coverImage
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

<div class="flex flex-col justify-between items-center p-8 h-full">
	<div class="max-w-sm">
		<img src={$coverImage} alt={$entityTitle} class="object-contain rounded-lg shadow-lg" />
	</div>

	<div class="w-full text-center">
		<div class="w-full text-xl truncate">
			{$entityTitle || ''}
		</div>
		<div class="text-gray-400">
			{$currentTrack?.artist.name || ''}
		</div>
		<div class="text-base text-gray-500 xl:text-4xl">
			{$currentTrack.number} of {$numOfTracks}
		</div>
	</div>

	<div class="flex flex-col w-full text-lg text-center">
		<div class="flex flex-col gap-y-4 mx-auto w-full text-2xl">
			<div
				bind:offsetWidth={titleWrapperWidth}
				class:justify-center={!$enableMarquee}
				class="flex overflow-hidden flex-row"
			>
				<div
					class:marquee={$enableMarquee}
					class:pl-[50%]={$enableMarquee}
					class="flex flex-row py-2 font-semibold whitespace-nowrap md:py-4 xl:py-8 leading-[1.15em]"
				>
					<span bind:offsetWidth={titleWidth}>
						{$currentTrack?.title || ''}
					</span>
				</div>

				{#if $enableMarquee}
					<div
						class:marquee={$enableMarquee}
						class:pl-[50%]={$enableMarquee}
						class="flex flex-row py-2 font-semibold whitespace-nowrap md:py-4 xl:py-8 leading-[1.15em]"
					>
						{$currentTrack?.title || ''}
					</div>
				{/if}
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

		<div class="flex flex-row flex-grow gap-2 justify-center h-10">
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
