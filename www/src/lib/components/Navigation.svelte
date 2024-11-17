<script>
	import Button from './Button.svelte';
	import { currentStatus } from '$lib/websocket';

	export let toggleList, controls;
	import {
		MenuIcon,
		SkipBackIcon,
		SkipForwardIcon,
		PlayIcon,
		PauseIcon
	} from 'svelte-feather-icons';
</script>

<div class="flex flex-row gap-x-4 md:mt-0 px-4 md:px-12 items-end justify-between">
	<div class="flex flex-row justify-center gap-x-4 flex-grow">
		<Button onClick={() => controls?.previous()}><SkipBackIcon /></Button>
		<Button onClick={() => controls?.playPause()}>
			{#if $currentStatus === 'Playing'}
				<PauseIcon />
			{:else}
				<PlayIcon />
			{/if}
		</Button>
		<Button onClick={() => controls?.next()}><SkipForwardIcon /></Button>
		<span class="z-20">
			<Button onClick={toggleList}><MenuIcon /></Button>
		</span>
	</div>
</div>
